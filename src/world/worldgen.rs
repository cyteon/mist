use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::world::chunks::{Chunk, Region};

// world variable accessible everywhere
pub static REGIONS: Lazy<Mutex<HashMap<(i32, i32), Arc<Mutex<Region>>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub async fn initial_gen() {
    let start_time = std::time::Instant::now();
    crate::log::log(fancy_log::LogLevel::Info, "Generating world...");

    for x in -1..=0 {
        for z in -1..=0 {
            let mut region = Region::new(x, z);

            for cx in 0..32 {
                for cz in 0..32 {
                    region.chunks.push(Chunk::generate((x << 5) + cx, (z << 5) + cz));
                }
            }

            let mut regions = REGIONS.lock().await;
            regions.insert((x, z), Arc::new(Mutex::new(region)));
        }
    }

    let duration = start_time.elapsed();
    crate::log::log(fancy_log::LogLevel::Info, format!("World generated in {:.2?}", duration).as_str());
}

// only loading into memory once needed helps reduce memory usage, we will also unload later on
pub async fn get_region(x: i32, z: i32) -> Arc<Mutex<Region>> {
    let mut regions = REGIONS.lock().await;
    let region = regions.get(&(x, z));

    match region {
        Some(r) => Arc::clone(r),

        None => {
            let region_file_path = format!("world/regions/{}_{}.mist_region", x, z);
            
            if std::path::Path::new(&region_file_path).exists() {
                let region = Region::load(x, z).await.ok().unwrap();
                let region_arc = Arc::new(Mutex::new(region));
                regions.insert((x, z), Arc::clone(&region_arc));

                region_arc
            } else {
                let mut region = Region::new(x, z);

                for cx in 0..32 {
                    for cz in 0..32 {
                        region.chunks.push(Chunk::generate((x << 5) + cx, (z << 5) + cz));
                    }
                }

                dbg!("Generated new region {}, {}", x, z);

                let region_arc = Arc::new(Mutex::new(region));
                regions.insert((x, z), Arc::clone(&region_arc));

                region_arc
            }
        },
    }
}