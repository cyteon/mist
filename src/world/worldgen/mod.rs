pub mod foliage;
pub mod noise;
pub mod ores;

use crate::world::worldgen::noise::trilerp;

use super::chunks::{Chunk, Region, Section};
use noise::NoiseFn;
use rayon::prelude::*;

pub const SEA_LEVEL: i32 = 62;
pub const MIN_Y: i32 = -64;
pub const MAX_Y: i32 = 319;

const COLS: usize = (16 / 4) as usize + 1;
const ROWS: usize = ((MAX_Y + 1 - MIN_Y) / 8) as usize + 1;

pub async fn initial_gen() {
    let start_time = std::time::Instant::now();
    crate::log::log(fancy_log::LogLevel::Info, "Generating world...");

    for x in -1..=0 {
        for z in -1..=0 {
            let mut region = Region::new(x, z);

            region.chunks = (0..32 * 32)
                .into_par_iter()
                .map(|i| {
                    let cx = i % 32;
                    let cz = i / 32;

                    generate((x << 5) + cx, (z << 5) + cz)
                })
                .collect();

            region.save().await.unwrap();
            crate::log::log(
                fancy_log::LogLevel::Info,
                &format!("Generated region {}, {}", x, z),
            );
        }
    }

    let duration = start_time.elapsed();
    crate::log::log(
        fancy_log::LogLevel::Info,
        format!("World generated in {:.2?}", duration).as_str(),
    );
}

pub fn generate(x: i32, z: i32) -> Chunk {
    let mut chunk = Chunk {
        x,
        z,

        sections: (0..24).map(|y| Section::new(y)).collect(),
    };

    let wx0 = x << 4;
    let wz0 = z << 4;

    let mut shapes = [[(0.0, 0.0); COLS]; COLS];

    for i in 0..COLS {
        for j in 0..COLS {
            shapes[i][j] =
                noise::sample_shape((wx0 + i as i32 * 4) as f64, (wz0 + j as i32 * 4) as f64);
        }
    }

    let mut lattice = [[[(0.0, 0.0); COLS]; COLS]; ROWS];

    for r in 0..ROWS {
        let y = (MIN_Y + r as i32 * 8) as f64;

        for i in 0..COLS {
            for j in 0..COLS {
                lattice[r][i][j] = density(
                    (wx0 + i as i32 * 4) as f64,
                    y,
                    (wz0 + j as i32 * 4) as f64,
                    shapes[i][j].0,
                    shapes[i][j].1,
                )
            }
        }
    }

    let mut heights = [[MIN_Y; 16]; 16];
    let mut terrain_heights = [[MIN_Y; 16]; 16];

    for r in 0..ROWS - 1 {
        for i in 0..COLS - 1 {
            for j in 0..COLS - 1 {
                let d000 = lattice[r][i][j];
                let d100 = lattice[r][i + 1][j];
                let d001 = lattice[r][i][j + 1];
                let d101 = lattice[r][i + 1][j + 1];
                let d010 = lattice[r + 1][i][j];
                let d110 = lattice[r + 1][i + 1][j];
                let d011 = lattice[r + 1][i][j + 1];
                let d111 = lattice[r + 1][i + 1][j + 1];

                for by in 0..8 {
                    let ty = by as f64 / 8.0;
                    let y = MIN_Y + r as i32 * 8 + by;

                    for bx in 0..4 {
                        let tx = bx as f64 / 4.0;
                        let lx = (i * 4 + bx) as u8;

                        for bz in 0..4 {
                            let tz = bz as f64 / 4.0;
                            let lz = (j * 4 + bz) as u8;

                            let t = trilerp(
                                d000.0, d100.0, d001.0, d101.0, d010.0, d110.0, d011.0, d111.0, tx,
                                ty, tz,
                            );

                            if t <= 0.0 {
                                continue;
                            }

                            if y > terrain_heights[lx as usize][lz as usize] {
                                terrain_heights[lx as usize][lz as usize] = y;
                            }

                            let d = trilerp(
                                d000.1, d100.1, d001.1, d101.1, d010.1, d110.1, d011.1, d111.1, tx,
                                ty, tz,
                            );

                            if d > 0.0 {
                                let block = if is_deepslate(wx0 + lx as i32, y, wz0 + lz as i32) {
                                    crate::types::blocks::DEEPSLATE
                                } else {
                                    crate::types::blocks::STONE
                                };

                                chunk.set_block(lx, y, lz, block);

                                if y > heights[lx as usize][lz as usize] {
                                    heights[lx as usize][lz as usize] = y;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    apply_surface(&mut chunk, &heights);
    fill_fluids(&mut chunk, &terrain_heights);
    ores::place_ores(&mut chunk);
    foliage::place_foliage(&mut chunk, &heights);

    chunk
}

fn fill_fluids(chunk: &mut Chunk, heights: &[[i32; 16]; 16]) {
    for x in 0..16u8 {
        for z in 0..16u8 {
            let h = heights[x as usize][z as usize];

            for y in (h + 1)..=SEA_LEVEL {
                chunk.set_block(x, y, z, crate::types::blocks::WATER);
            }
        }
    }
}

fn density(wx: f64, y: f64, wz: f64, offset: f64, factor: f64) -> (f64, f64) {
    let grad = if y < offset {
        (offset - y) * 0.8
    } else {
        (offset - y) * 1.4
    };

    let n = noise::fbm3(&noise::DENSITY, wx, y * 2.0, wz, 4, 1.0 / 128.0);
    let d = grad + n * factor;

    if d <= 0.0 {
        return (d, d);
    }

    (d, d.min(cave_density(wx, y, wz, offset, factor)))
}

fn cave_density(wx: f64, y: f64, wz: f64, offset: f64, factor: f64) -> f64 {
    let depth = offset - factor - y;

    if depth < -2.0 {
        return 100.0;
    }

    let entrance = ((noise::ENTRANCES.get([wx / 128.0, wz / 128.0]) - 0.25) / 2.0).clamp(0.0, 1.0);
    let land = ((offset - (SEA_LEVEL as f64 + 4.0)) / 6.0).clamp(0.0, 1.0);
    let spaghetti_min = 9.0 * (1.0 - entrance * land);

    let a = noise::SPAGHETTI_A.get([wx / 96.0, y / 48.0, wz / 96.0]);
    let b = noise::SPAGHETTI_B.get([wx / 96.0, y / 48.0, wz / 96.0]);
    let spaghetti = (a * a + b * b) * 280.0 - 5.5 + ((spaghetti_min - depth) * 4.0).max(0.0);

    let c = noise::fbm3(&noise::CHEESE, wx, y * 2.2, wz, 4, 1.0 / 96.0);
    let grow = (depth / 200.0).clamp(0.0, 0.1);
    let cheese = (0.42 - grow - c) * 60.0 + ((16.0 - depth) * 4.0).max(0.0);

    let floor = ((-59.0 - y) * 4.0).max(0.0);

    spaghetti.min(cheese) + floor
}

pub fn surface_height(wx: i32, wz: i32) -> i32 {
    let x0 = wx.div_euclid(4) * 4;
    let z0 = wz.div_euclid(4) * 4;
    let tx = (wx - x0) as f64 / 4.0;
    let tz = (wz - z0) as f64 / 4.0;

    let shapes = [
        noise::sample_shape(x0 as f64, z0 as f64),
        noise::sample_shape((x0 + 4) as f64, z0 as f64),
        noise::sample_shape(x0 as f64, (z0 + 4) as f64),
        noise::sample_shape((x0 + 4) as f64, (z0 + 4) as f64),
    ];

    let sample = |si: usize, dx: i32, dz: i32, y: i32| {
        density(
            (x0 + dx) as f64,
            y as f64,
            (z0 + dz) as f64,
            shapes[si].0,
            shapes[si].1,
        )
        .1
    };

    for r in (0..ROWS - 1).rev() {
        let y0 = MIN_Y + r as i32 * 8;
        let y1 = y0 + 8;

        let d000 = sample(0, 0, 0, y0);
        let d100 = sample(1, 4, 0, y0);
        let d001 = sample(2, 0, 4, y0);
        let d101 = sample(3, 4, 4, y0);
        let d010 = sample(0, 0, 0, y1);
        let d110 = sample(1, 4, 0, y1);
        let d011 = sample(2, 0, 4, y1);
        let d111 = sample(3, 4, 4, y1);

        for by in (0..8).rev() {
            let ty = by as f64 / 8.0;
            let d = trilerp(d000, d100, d001, d101, d010, d110, d011, d111, tx, ty, tz);

            if d > 0.0 {
                return y0 + by;
            }
        }
    }

    MIN_Y
}

fn apply_surface(chunk: &mut Chunk, heights: &[[i32; 16]; 16]) {
    for x in 0..16u8 {
        for z in 0..16u8 {
            chunk.set_block(x, MIN_Y, z, crate::types::blocks::BEDROCK);

            let h = heights[x as usize][z as usize];

            if h <= MIN_Y {
                continue;
            }

            let beach = h <= SEA_LEVEL + 1;

            let top = if beach {
                crate::types::blocks::SAND
            } else {
                crate::types::blocks::GRASS_BLOCK
            };

            let under = if beach {
                crate::types::blocks::SAND
            } else {
                crate::types::blocks::DIRT
            };

            chunk.set_block(x, h, z, top);

            for y in (h - 3)..h {
                if y <= MIN_Y {
                    continue;
                }

                if chunk.get_block(x, y, z) != crate::types::blocks::AIR {
                    chunk.set_block(x, y, z, under);
                }
            }
        }
    }
}

fn is_deepslate(x: i32, y: i32, z: i32) -> bool {
    if y >= 0 {
        return false;
    }

    if y < -8 {
        return true;
    }

    ((hash3(x, y, z) & 0xff) as i32) < -y * 32
}

fn hash3(x: i32, y: i32, z: i32) -> u64 {
    let mut h = (x as u64).wrapping_mul(0x9E3779B97F4A7C15)
        ^ (y as u64).wrapping_mul(0xC2B2AE3D27D4EB4F)
        ^ (z as u64).wrapping_mul(0x165667B19E3779F9);

    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58476D1CE4E5B9);
    h ^ (h >> 32)
}
