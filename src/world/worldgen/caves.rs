use crate::world::chunks::Chunk;
use noise::NoiseFn;

pub fn carve_caves(chunk: &mut Chunk, cave_tops: &[[i32; 16]; 16]) {
    let cx = (chunk.x << 4) as f64;
    let cz = (chunk.z << 4) as f64;

    for bx in 0..16 {
        for bz in 0..16 {
            let ct = cave_tops[bx as usize][bz as usize];

            for y in -60..=ct {
                if chunk.get_block(bx as u8, y, bz as u8) == 0 {
                    continue;
                }

                let v1 = super::noise::PERLIN.get([
                    (cx + bx as f64) / 64.0,
                    (y as f64) / 64.0,
                    (cz + bz as f64) / 64.0,
                ]);
                let v2 = super::noise::PERLIN.get([
                    (cx + bx as f64) / 64.0 + 100.0,
                    (y as f64) / 64.0 + 100.0,
                    (cz + bz as f64) / 64.0 + 100.0,
                ]);

                if v1 * v1 + v2 * v2 < 0.005 {
                    chunk.set_block(bx as u8, y, bz as u8, crate::types::blocks::CAVE_AIR);
                }
            }
        }
    }
}
