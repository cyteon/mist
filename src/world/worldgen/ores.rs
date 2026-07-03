use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::world::chunks::Chunk;

enum Height {
    Uniform(i32, i32),
    Triangle(i32, i32),
}

struct OreConfig {
    block: u32,
    count: i32,
    size: i32,
    height: Height,
}

const ORES: &[OreConfig] = &[
    OreConfig {
        block: crate::types::blocks::COAL_ORE as u32,
        count: 30,
        size: 17,
        height: Height::Uniform(136, 320),
    },
    OreConfig {
        block: crate::types::blocks::COAL_ORE as u32,
        count: 20,
        size: 17,
        height: Height::Triangle(0, 192),
    },
    OreConfig {
        block: crate::types::blocks::IRON_ORE as u32,
        count: 90,
        size: 9,
        height: Height::Triangle(80, 384),
    },
    OreConfig {
        block: crate::types::blocks::IRON_ORE as u32,
        count: 10,
        size: 9,
        height: Height::Triangle(-24, 56),
    },
    OreConfig {
        block: crate::types::blocks::IRON_ORE as u32,
        count: 10,
        size: 4,
        height: Height::Uniform(-64, 72),
    },
    OreConfig {
        block: crate::types::blocks::COPPER_ORE as u32,
        count: 16,
        size: 10,
        height: Height::Triangle(-16, 112),
    },
    OreConfig {
        block: crate::types::blocks::GOLD_ORE as u32,
        count: 4,
        size: 9,
        height: Height::Triangle(-64, 32),
    },
    OreConfig {
        block: crate::types::blocks::GOLD_ORE as u32,
        count: 1,
        size: 9,
        height: Height::Uniform(-64, -48),
    },
    OreConfig {
        block: crate::types::blocks::REDSTONE_ORE as u32,
        count: 4,
        size: 8,
        height: Height::Uniform(-64, 15),
    },
    OreConfig {
        block: crate::types::blocks::REDSTONE_ORE as u32,
        count: 8,
        size: 8,
        height: Height::Triangle(-96, -32),
    },
    OreConfig {
        block: crate::types::blocks::DIAMOND_ORE as u32,
        count: 7,
        size: 4,
        height: Height::Triangle(-144, 16),
    },
    OreConfig {
        block: crate::types::blocks::DIAMOND_ORE as u32,
        count: 4,
        size: 8,
        height: Height::Uniform(-64, -4),
    },
    OreConfig {
        block: crate::types::blocks::LAPIS_ORE as u32,
        count: 2,
        size: 7,
        height: Height::Triangle(-32, 32),
    },
    OreConfig {
        block: crate::types::blocks::LAPIS_ORE as u32,
        count: 4,
        size: 7,
        height: Height::Uniform(-64, 64),
    },
];

pub fn place_ores(chunk: &mut Chunk) {
    let seed = chunk.chunk_seed().wrapping_add(0x0DE5);
    let mut rng = SmallRng::seed_from_u64(seed);

    for ore in ORES {
        for _ in 0..ore.count {
            let x = rng.gen_range(0..16) as u8;
            let z = rng.gen_range(0..16) as u8;
            let y = sample_height(&mut rng, &ore.height);

            if !(super::MIN_Y..=super::MAX_Y).contains(&y) {
                continue;
            }

            place_vein(chunk, &mut rng, x as i32, y, z as i32, ore);
        }
    }
}

fn sample_height(rng: &mut SmallRng, h: &Height) -> i32 {
    match h {
        Height::Uniform(min, max) => rng.gen_range(*min..=*max),
        Height::Triangle(min, max) => {
            let half = (max - min) / 2;
            min + rng.gen_range(0..=half) + rng.gen_range(0..=half)
        }
    }
}

fn place_vein(chunk: &mut Chunk, rng: &mut SmallRng, x: i32, y: i32, z: i32, ore: &OreConfig) {
    let size = ore.size as f64;
    let angle = rng.r#gen::<f64>() * std::f64::consts::PI;
    let len = size / 8.0;

    let x1 = x as f64 + angle.sin() * len;
    let x2 = x as f64 - angle.sin() * len;
    let z1 = z as f64 + angle.cos() * len;
    let z2 = z as f64 - angle.cos() * len;
    let y1 = y as f64 + rng.gen_range(-2.0..=2.0);
    let y2 = y as f64 + rng.gen_range(-2.0..=2.0);

    let steps = ore.size.max(1) as usize;

    for i in 0..steps {
        let t = i as f64 / steps as f64;

        let cx = x1 + (x2 - x1) * t;
        let cy = y1 + (y2 - y1) * t;
        let cz = z1 + (z2 - z1) * t;

        let d = rng.r#gen::<f64>() * size / 16.0;
        let radius = (((t * std::f64::consts::PI).sin() + 1.0) * d + 1.0) / 2.0;

        let bx0 = (cx - radius).floor() as i32;
        let bx1 = (cx + radius).floor() as i32;
        let by0 = (cy - radius).floor() as i32;
        let by1 = (cy + radius).floor() as i32;
        let bz0 = (cz - radius).floor() as i32;
        let bz1 = (cz + radius).floor() as i32;

        for bx in bx0..=bx1 {
            let dx = (bx as f64 + 0.5 - cx) / radius;
            if dx * dx >= 1.0 {
                continue;
            }

            for by in by0..=by1 {
                let dy = (by as f64 + 0.5 - cy) / radius;
                if dx * dx + dy * dy >= 1.0 {
                    continue;
                }

                for bz in bz0..=bz1 {
                    let dz = (bz as f64 + 0.5 - cz) / radius;
                    if dx * dx + dy * dy + dz * dz >= 1.0 {
                        continue;
                    }

                    try_place(chunk, bx, by, bz, ore);
                }
            }
        }
    }
}

fn try_place(chunk: &mut Chunk, x: i32, y: i32, z: i32, ore: &OreConfig) {
    if !(0..16).contains(&x) || !(0..16).contains(&z) || !(super::MIN_Y..=super::MAX_Y).contains(&y)
    {
        return;
    }

    let (x, z) = (x as u8, z as u8);
    let current = chunk.get_block(x, y, z);

    if current != crate::types::blocks::STONE && current != crate::types::blocks::DEEPSLATE {
        return;
    }

    let block = if current == crate::types::blocks::DEEPSLATE {
        crate::types::blocks::deepslate_variant(ore.block as u16)
    } else {
        ore.block as u16
    };

    chunk.set_block(x, y, z, block);
}
