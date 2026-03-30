pub mod noise;
pub mod ores;

use rayon::prelude::*;

use super::chunks::{Chunk, Section, Region};

const SEA_LEVEL: i32 = 62;

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
            crate::log::log(fancy_log::LogLevel::Info, &format!("Generated region {}, {}", x, z));
        }
    }

    let duration = start_time.elapsed();
    crate::log::log(fancy_log::LogLevel::Info, format!("World generated in {:.2?}", duration).as_str());
}

pub fn generate(x: i32, z: i32) -> Chunk {
    let mut chunk = Chunk {
        x,
        z,

        sections: (0..24).map(|y| Section::new(y)).collect(),
    };

    for x in 0..16 {
        for z in 0..16 {
            let wx = (chunk.x << 4) + x;
            let wz = (chunk.z << 4) + z;

            let height = noise::get_height(&noise::PERLIN, wx as f64, wz as f64);
            place_column(&mut chunk, x as u8, z as u8, height);
        }
    }

    ores::place_ores(&mut chunk);

    chunk
}

fn place_column(chunk: &mut Chunk, x: u8, z: u8, height: i32) {
    chunk.set_block(x, -64, z, crate::types::blocks::BEDROCK);

    for y in -63..=height {
        let block_id = match y {
            y if y >= SEA_LEVEL && y == height => crate::types::blocks::GRASS_BLOCK,
            y if y < SEA_LEVEL && y == height => crate::types::blocks::SAND,
            y if y < SEA_LEVEL && y > height - 4 => crate::types::blocks::SAND,
            y if y > height - 4 => crate::types::blocks::DIRT,
            _ => crate::types::blocks::STONE,
        };

        chunk.set_block(x, y, z, block_id);
    }

    if height < SEA_LEVEL {
        for y in (height + 1)..SEA_LEVEL {
            chunk.set_block(x, y, z, crate::types::blocks::WATER);
        }
    }
}