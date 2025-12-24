use std::collections::HashMap;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::world::chunks::{Chunk, Region};

// world variable accessible everywhere
pub static REGIONS: Lazy<Mutex<HashMap<(i32, i32), Region>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub async fn initial_gen() {
    let start_time = std::time::Instant::now();
    fancy_log::log(fancy_log::LogLevel::Info, "Generating world...");

    let mut region = Region::new(0, 0);

    for x in 0..32 {
        for z in 0..32 {
            region.chunks.push(Chunk::generate(x, z));
        }
    }

    let mut regions = REGIONS.lock().await;
    regions.insert((0, 0), region);


    let duration = start_time.elapsed();
    fancy_log::log(fancy_log::LogLevel::Info, format!("World generated in {:.2?}", duration).as_str());
}