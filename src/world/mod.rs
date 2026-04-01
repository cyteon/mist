pub mod chunks;
pub mod worldgen;

use std::{collections::HashMap, sync::Arc};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use rayon::prelude::*;

use chunks::Region;

pub static REGIONS: Lazy<Mutex<HashMap<(i32, i32), Arc<Mutex<Region>>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub async fn get_region(x: i32, z: i32) -> Arc<Mutex<Region>> {
    {
        let regions = REGIONS.lock().await;
        if let Some(region) = regions.get(&(x, z)) {
            return Arc::clone(region);
        }
    }

    let region_file_path = format!("world/regions/{}_{}.mist_region", x, z);
    
    let region = if std::path::Path::new(&region_file_path).exists() {
        Region::load(x, z).await.ok().unwrap()
    } else {
        println!("Generating region {}, {}", x, z);
        
        tokio::task::spawn_blocking(move || {
            let mut region = Region::new(x, z);

            region.chunks = (0..32 * 32)
                .into_par_iter()
                .map(|i| {
                    let cx = i % 32;
                    let cz = i / 32;

                    println!("Generating chunk {}, {} in region {}, {}", cx, cz, x, z);

                    worldgen::generate((x << 5) + cx, (z << 5) + cz)
                })
                .collect();

            region
        }).await.unwrap()
    };

    println!("Region {}, {} loaded", x, z);

    let region_arc = Arc::new(Mutex::new(region));
        
    let mut regions = REGIONS.lock().await;
    regions.insert((x, z), Arc::clone(&region_arc));

    region_arc
}