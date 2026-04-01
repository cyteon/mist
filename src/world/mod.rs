pub mod chunks;
pub mod worldgen;

use std::{collections::HashMap, sync::Arc};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use chunks::{Chunk, Region};

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
        Region::new(x, z)
    };

    let region_arc = Arc::new(Mutex::new(region));

    let mut regions = REGIONS.lock().await;
    regions.insert((x, z), Arc::clone(&region_arc));

    region_arc
}

pub async fn get_chunk(region: &Arc<Mutex<Region>>, cx: i32, cz: i32) -> Chunk {
    {
        let region_guard = region.lock().await;
        if let Some(chunk) = region_guard.get_chunk(cx, cz) {
            return chunk.clone();
        }
    }

    let chunk = tokio::task::spawn_blocking(move || {
        worldgen::generate(cx, cz)
    }).await.unwrap();

    let mut region_guard = region.lock().await;
    if let Some(existing) = region_guard.get_chunk(cx, cz) {
        return existing.clone();
    }
    region_guard.chunks.push(chunk.clone());
    chunk
}