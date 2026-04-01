use rand::{SeedableRng, Rng};

use crate::world::chunks::Chunk;

// TODO: minecraft uses both uniform triangle distribution, so gotta implement that

struct OreConfig {
    block: u32,
    veins_per_chunk: i32,
    max_size: i32,
    min_y: i32,
    max_y: i32,
}

const ORES: &[OreConfig] = &[
    OreConfig { block: crate::types::blocks::COAL_ORE as u32, veins_per_chunk: 30, max_size: 16, min_y: 0, max_y: 320 },

    OreConfig { block: crate::types::blocks::IRON_ORE as u32, veins_per_chunk: 10, max_size: 9, min_y: -64, max_y: 320 },

    OreConfig { block: crate::types::blocks::COPPER_ORE as u32, veins_per_chunk: 16, max_size: 10, min_y: -16, max_y: 112 },

    OreConfig { block: crate::types::blocks::GOLD_ORE as u32, veins_per_chunk: 4, max_size: 9, min_y: -64, max_y: 32 },

    OreConfig { block: crate::types::blocks::REDSTONE_ORE as u32, veins_per_chunk: 4, max_size: 8, min_y: -64, max_y: -32 },
    OreConfig { block: crate::types::blocks::REDSTONE_ORE as u32, veins_per_chunk: 4, max_size: 8, min_y: -64, max_y: 15 },

    OreConfig { block: crate::types::blocks::DIAMOND_ORE as u32, veins_per_chunk: 3, max_size: 8, min_y: -64, max_y: 16 },

    OreConfig { block: crate::types::blocks::LAPIS_ORE as u32, veins_per_chunk: 2, max_size: 7, min_y: -64, max_y: 64 },
];

pub fn place_ores(chunk: &mut Chunk) {
    let seed = chunk.chunk_seed();
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);

    for ore in ORES {
        for _ in 0..ore.veins_per_chunk {
            let vein_size = rng.gen_range(1..=ore.max_size);
            
            let x = rng.gen_range(0..16);
            let z = rng.gen_range(0..16);
            let y = rng.gen_range(ore.min_y..ore.max_y);

            if chunk.get_block(x, y, z) != crate::types::blocks::STONE {
                continue;
            }

            for _ in 0..=vein_size {
                let offset_x = rng.gen_range(-2..=2);
                let offset_y = rng.gen_range(-2..=2);
                let offset_z = rng.gen_range(-2..=2);

                let vx = (x as i32 + offset_x).clamp(0, 15) as u8;
                let vy = (y + offset_y).clamp(-64, 319) as i32;
                let vz = (z as i32 + offset_z).clamp(0, 15) as u8;

                chunk.set_block(vx, vy, vz, ore.block as u16);
            }
        }
    }
}