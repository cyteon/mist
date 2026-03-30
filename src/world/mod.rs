pub mod chunks;
pub mod worldgen;

use std::{collections::HashMap, sync::Arc};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use chunks::Region;

pub static REGIONS: Lazy<Mutex<HashMap<(i32, i32), Arc<Mutex<Region>>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

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
                        region.chunks.push(worldgen::generate((x << 5) + cx, (z << 5) + cz));
                    }
                }

                let region_arc = Arc::new(Mutex::new(region));
                regions.insert((x, z), Arc::clone(&region_arc));

                region_arc
            }
        },
    }
}