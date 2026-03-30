use super::chunks::{Chunk, Section, Region};

pub async fn initial_gen() {
    let start_time = std::time::Instant::now();
    crate::log::log(fancy_log::LogLevel::Info, "Generating world...");

    for x in -1..=0 {
        for z in -1..=0 {
            let mut region = Region::new(x, z);

            for cx in 0..32 {
                for cz in 0..32 {
                    region.chunks.push(generate((x << 5) + cx, (z << 5) + cz));
                }
            }

            region.save().await.unwrap();
            crate::log::log(fancy_log::LogLevel::Info, &format!("Generated region {}, {}", x, z));
        }
    }

    let duration = start_time.elapsed();
    crate::log::log(fancy_log::LogLevel::Info, format!("World generated in {:.2?}", duration).as_str());
}

pub fn generate(x: i32, z: i32) -> Chunk {
    // we will use this when proper generation
    let _seed = crate::config::SERVER_CONFIG.world_seed as u64;

    // TODO: actual generation

    let mut chunk = Chunk {
        x,
        z,

        sections: (0..24).map(|y| Section::new(y)).collect(),
    };

    for x in 0..16 {
        for z in 0..16 {
            chunk.sections[0].set_block(
                x, 0, z, 
                crate::types::blocks::get("minecraft:grass_block").unwrap().id
            );
        }
    }

    chunk
}