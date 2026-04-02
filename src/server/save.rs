use fancy_log::LogLevel;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PlayerSave {
    pub uuid: String,

    pub inventory: Vec<Option<crate::types::items::ItemStack>>,

    pub is_op: bool,
    pub gamemode: crate::types::player::Gamemode,

    pub health: f32,
    pub hunger: i32,
    pub saturation: f32,

    pub x: f64,
    pub y: f64,
    pub z: f64,

    pub vx: f64,
    pub vy: f64,
    pub vz: f64,

    pub yaw: f32,
    pub pitch: f32,
}

impl Default for PlayerSave {
    fn default() -> Self {
        Self {
            uuid: String::new(),
            inventory: vec![None; 36],

            is_op: false,
            gamemode: match crate::config::SERVER_CONFIG.default_gamemode.as_str() {
                "survival" => crate::types::player::Gamemode::Survival,
                "creative" => crate::types::player::Gamemode::Creative,
                "adventure" => crate::types::player::Gamemode::Adventure,
                "spectator" => crate::types::player::Gamemode::Spectator,
                _ => {
                    crate::log::log(fancy_log::LogLevel::Warn, format!("Invalid default gamemode: {}, defaulting to survival", crate::config::SERVER_CONFIG.default_gamemode).as_str());
                    crate::types::player::Gamemode::Survival
                }
            },

            health: 20.0,
            hunger: 20,
            saturation: 5.0,

            x: 0.0,
            y: 0.0,
            z: 0.0,

            vx: 0.0,
            vy: 0.0,
            vz: 0.0,

            yaw: 0.0,
            pitch: 0.0,
        }
    }
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
        save_player(&player).await;
    }

    for region in crate::world::REGIONS.lock().await.values() {
        let region = region.lock().await;
        let _ = region.save().await;
    }

    let duration = start.elapsed();
    crate::log::log(LogLevel::Info, format!("Save complete in {:.2?}", duration).as_str());
}

pub async fn save_player(player: &crate::types::player::Player) {
    let inventory = player.inventory.iter().map(|slot| slot.clone()).collect();

    let player_save = PlayerSave {
        uuid: player.uuid.clone(),
        inventory,

        is_op: player.is_op,
        gamemode: player.gamemode,

        health: player.health,
        hunger: player.hunger,
        saturation: player.saturation,

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

pub fn load_player(uuid: &str) -> Option<PlayerSave> {
    let player_path = format!(
        "{}/players/{}.json", 
        crate::config::SERVER_CONFIG.world_name.clone(), 
        uuid
    );

    if !std::path::Path::new(&player_path).exists() {
        return None;
    }

    let player_json = std::fs::read_to_string(player_path).unwrap();
    let player_save: PlayerSave = serde_json::from_str(&player_json).unwrap();

    Some(player_save)
}