use fancy_log::LogLevel;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PlayerSave {
    pub uuid: String,

    pub x: f64,
    pub y: f64,
    pub z: f64,

    pub vx: f64,
    pub vy: f64,
    pub vz: f64,

    pub yaw: f32,
    pub pitch: f32,
}

pub fn ensure_save_folders() {
    std::fs::create_dir_all(crate::config::SERVER_CONFIG.world_name.clone()).unwrap();
    std::fs::create_dir_all(format!("{}/players", crate::config::SERVER_CONFIG.world_name.clone())).unwrap();
    std::fs::create_dir_all(format!("{}/regions", crate::config::SERVER_CONFIG.world_name.clone())).unwrap();
}

pub fn exists(path: &str) -> bool {
    std::path::Path::new(format!(
        "{}/{}",
        crate::config::SERVER_CONFIG.world_name.clone(),
        path
    ).as_str()).exists()
}

pub async fn save() {
    ensure_save_folders();
    fancy_log::log(LogLevel::Info, "Autosaving...");

    let start = std::time::Instant::now();

    for player in crate::server::state::play::PLAYERS.read().await.values() {
        let player = player.lock().await;

        let player_save = PlayerSave {
            uuid: player.uuid.clone(),

            x: player.x,
            y: player.y,
            z: player.z,

            vx: player.vx,
            vy: player.vy,
            vz: player.vz,

            yaw: player.yaw,
            pitch: player.pitch,
        };
        
        let player_json = serde_json::to_string_pretty(&player_save).unwrap();
        let player_path = format!(
            "{}/players/{}.json", 
            crate::config::SERVER_CONFIG.world_name.clone(), 
            player.uuid
        );

        std::fs::write(player_path, player_json).unwrap();
    }

    for region in crate::world::worldgen::REGIONS.lock().await.values() {
        if region.save().await.is_err() {
            fancy_log::log(LogLevel::Error, format!(
                "Failed to save region {}, {}", 
                region.x, 
                region.z
            ).as_str());
        }
    }

    let duration = start.elapsed();
    fancy_log::log(LogLevel::Info, format!("Autosave complete in {:.2?}", duration).as_str());
}

pub async fn load_world() {
    ensure_save_folders();
    fancy_log::log(LogLevel::Info, "Loading world...");

    let start = std::time::Instant::now();

    let mut regions_lock = crate::world::worldgen::REGIONS.lock().await;

    let region_files = std::fs::read_dir(format!(
        "{}/regions", 
        crate::config::SERVER_CONFIG.world_name.clone()
    )).unwrap();

    for entry in region_files {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_str().unwrap();

        if file_name_str.ends_with(".mist_region") {
            let coords: Vec<&str> = file_name_str.trim_end_matches(".mist_region").split('_').collect();
            if coords.len() != 2 {
                continue;
            }

            let x: i32 = coords[0].parse().unwrap();
            let z: i32 = coords[1].parse().unwrap();

            match crate::world::chunks::Region::load(x, z).await {
                Ok(region) => {
                    regions_lock.insert((x, z), region);
                },
                
                Err(e) => {
                    fancy_log::log(LogLevel::Error, format!(
                        "Failed to load region {}, {}: {}", 
                        x, 
                        z, 
                        e
                    ).as_str());
                }
            }
        }
    }

    let duration = start.elapsed();
    fancy_log::log(LogLevel::Info, format!("World loaded in {:.2?}", duration).as_str());
}