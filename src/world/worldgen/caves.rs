use noise::NoiseFn;
use crate::world::chunks::Chunk;

pub fn carve_caves(chunk: &mut Chunk, cave_tops: &[[i32; 16]; 16]) {
    let cx = chunk.x << 4;
    let cz = chunk.z << 4;

    for x in 0..16u8 {
        for z in 0..16u8 {
            let wx = ((chunk.x << 4) + x as i32) as f64;
            let wz = ((chunk.z << 4) + z as i32) as f64;
            let ct = cave_tops[x as usize][z as usize];

            for y in -60..=ct {
                if chunk.get_block(x, y, z) == crate::types::blocks::AIR {
                    continue;
                }

                if !might_have_cave(wx, y as f64, wz) {
                    continue;
                }

                if super::noise::is_cave(wx, y as f64, wz) {
                    chunk.set_block(x, y, z, crate::types::blocks::CAVE_AIR);
                }
            }
        }
    }
}

fn might_have_cave(x: f64, y: f64, z: f64) -> bool {
    let first = super::noise::PERLIN.get([x / 64.0, y / 64.0, z / 64.0]);

    if first > -0.15 {
        return true;
    }

    let v1 = super::noise::PERLIN.get([x / 64.0, y / 64.0, z / 64.0]);
    if v1.abs() > 0.1 {
        return false;
    }

    let v2 = super::noise::PERLIN.get([x / 64.0 + 100.0, y / 64.0 + 100.0, z / 64.0 + 100.0]);
    v2.abs() < 0.1
}