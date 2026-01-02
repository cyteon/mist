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
    crate::log::log(LogLevel::Info, "Saving...");

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
            crate::log::log(LogLevel::Error, format!(
                "Failed to save region {}, {}", 
                region.x, 
                region.z
            ).as_str());
        }
    }

    let duration = start.elapsed();
    crate::log::log(LogLevel::Info, format!("Save complete in {:.2?}", duration).as_str());
}