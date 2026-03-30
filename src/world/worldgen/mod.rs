pub mod noise;

use super::chunks::{Chunk, Section, Region};

const SEA_LEVEL: i32 = 62;

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
    let mut chunk = Chunk {
        x,
        z,

        sections: (0..24).map(|y| Section::new(y)).collect(),
    };

    let stone = crate::types::blocks::get("minecraft:stone").unwrap().id;

    for x in 0..16 {
        for z in 0..16 {
            let wx = (chunk.x << 4) + x;
            let wz = (chunk.z << 4) + z;

            let height = noise::get_height(&noise::PERLIN, wx as f64, wz as f64);
            place_column(&mut chunk, x as u8, z as u8, height, stone);
        }
    }

    chunk
}

fn place_column(chunk: &mut Chunk, x: u8, z: u8, height: i32, block_id: u16) {
    chunk.set_block(x, 0, z, crate::types::blocks::get("minecraft:bedrock").unwrap().id);
}