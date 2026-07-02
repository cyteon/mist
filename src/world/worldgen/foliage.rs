use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::world::Chunk;

const FLOWERS: [u16; 6] = [
    crate::types::blocks::DANDELION,
    crate::types::blocks::POPPY,
    crate::types::blocks::BLUE_ORCHID,
    crate::types::blocks::ALLIUM,
    crate::types::blocks::AZURE_BLUET,
    crate::types::blocks::RED_TULIP,
];

pub fn place_foliage(chunk: &mut Chunk, heights: &[[i32; 16]; 16]) {
    let seed = chunk.chunk_seed().wrapping_add(0xF01A);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);

    for x in 0..16u8 {
        for z in 0..16u8 {
            let h = heights[x as usize][z as usize];
            if h < super::SEA_LEVEL || h >= super::MAX_Y {
                continue;
            }

            if chunk.get_block(x, h, z) != crate::types::blocks::GRASS_BLOCK {
                continue;
            }

            if chunk.get_block(x, h + 1, z) != crate::types::blocks::AIR {
                continue;
            }

            if rng.gen_range(0..4) == 0 {
                chunk.set_block(x, h + 1, z, crate::types::blocks::SHORT_GRASS);
            } else if rng.gen_range(0..50) == 0 {
                chunk.set_block(x, h + 1, z, FLOWERS[rng.gen_range(0..FLOWERS.len())]);
            }
        }
    }

    let wx0 = chunk.x << 4;
    let wz0 = chunk.z << 4;

    for dx in -3..19i32 {
        for dz in -3..19i32 {
            let wx = wx0 + dx;
            let wz = wz0 + dz;

            let mut trng = SmallRng::seed_from_u64(tree_seed(wx, wz));
            if trng.gen_range(0..70) != 0 {
                continue;
            }

            let base = if (0..16).contains(&dx) && (0..16).contains(&dz) {
                heights[dx as usize][dz as usize]
            } else {
                super::surface_height(wx, wz)
            };

            if base <= super::SEA_LEVEL + 1 || base >= super::MAX_Y - 10 {
                continue;
            }

            place_oak(chunk, dx, base + 1, dz, &mut trng);
        }
    }
}

fn tree_seed(wx: i32, wz: i32) -> u64 {
    let mut h = (crate::config::SERVER_CONFIG.world_seed)
        ^ (wx as u64).wrapping_mul(0x9E3779B97F4A7C15)
        ^ (wz as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);

    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58476D1CE4E5B9);
    h ^ (h >> 31)
}

fn place_oak(chunk: &mut Chunk, x: i32, y: i32, z: i32, rng: &mut impl Rng) {
    let log_height = rng.gen_range(4..=6);

    set(chunk, x, y - 1, z, crate::types::blocks::DIRT, false);

    for dy in 0..log_height {
        set(chunk, x, y + dy, z, crate::types::blocks::OAK_LOG, false);
    }

    let leaf_base = y + log_height - 2;

    for dy in 0..4 {
        let radius: i32 = if dy < 2 { 2 } else { 1 };

        for lx in -radius..=radius {
            for lz in -radius..=radius {
                if lx == 0 && lz == 0 && dy < 2 {
                    continue;
                }

                let corner = lx.abs() == radius && lz.abs() == radius;

                if corner && dy >= 2 {
                    continue;
                }

                if corner && rng.gen_bool(0.5) {
                    continue;
                }

                set(
                    chunk,
                    x + lx,
                    leaf_base + dy,
                    z + lz,
                    crate::types::blocks::OAK_LEAVES,
                    true,
                );
            }
        }
    }
}

fn set(chunk: &mut Chunk, x: i32, y: i32, z: i32, block: u16, only_air: bool) {
    if !(0..16).contains(&x) || !(0..16).contains(&z) || !(super::MIN_Y..=super::MAX_Y).contains(&y)
    {
        return;
    }

    if only_air && chunk.get_block(x as u8, y, z as u8) != crate::types::blocks::AIR {
        return;
    }

    chunk.set_block(x as u8, y, z as u8, block);
}
