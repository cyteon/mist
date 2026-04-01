use rand::{Rng, SeedableRng};

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
    let seed = chunk.chunk_seed();
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);

    for x in 2..14 {
        for z in 2..14 {
            let h = heights[x as usize][z as usize];
            if h < super::SEA_LEVEL { continue; }

            if rng.gen_range(0..60) == 0 {
                let block = chunk.get_block(x, h, z);
                if block != crate::types::blocks::GRASS_BLOCK {
                    continue;
                }

                // todo, diffrent trees for biomes and stuff
                place_oak(chunk, x, h, z, &mut rng);
                continue;
            }

            if rng.gen_range(0..4) == 0 {
                let block = chunk.get_block(x, h, z);
                if block != crate::types::blocks::GRASS_BLOCK {
                    continue;
                }

                chunk.set_block(x, h + 1, z, crate::types::blocks::SHORT_GRASS);
                continue;
            }

            if rng.gen_range(0..50) == 0 {
                let block = chunk.get_block(x, h, z);
                if block != crate::types::blocks::GRASS_BLOCK {
                    continue;
                }

                chunk.set_block(x, h + 1, z, FLOWERS[rng.gen_range(0..FLOWERS.len())]);
                continue;
            }
        }
    }
}

fn place_oak(chunk: &mut Chunk, x: u8, y: i32, z: u8, rng: &mut impl Rng) {
    let log_height = rng.gen_range(4..=6);

    for dy in 0..log_height {
        chunk.set_block(x, y + dy, z, crate::types::blocks::OAK_LOG);
    }

    let leaf_base = y + log_height - 2;

    for dy in 0..3 {
        let radius = if dy == 2 { 1 } else { 2 };

        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx * dx + dz * dz > radius * radius + 1 { continue; }

                let lx = x as i32 + dx;
                let lz = z as i32 + dz;

                if lx < 0 || lx > 15 || lz < 0 || lz > 15 { continue; }

                if chunk.get_block(lx as u8, (leaf_base + dy) as i32, lz as u8) == crate::types::blocks::AIR {
                    chunk.set_block(lx as u8, leaf_base + dy, lz as u8, crate::types::blocks::OAK_LEAVES);
                }
            }
        }
    }
}