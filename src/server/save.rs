use crate::log::{self, LogLevel};
use std::sync::atomic::Ordering;

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
    pub dead: bool,

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
                    crate::log::log(
                        LogLevel::Warn,
                        format!(
                            "Invalid default gamemode: {}, defaulting to survival",
                            crate::config::SERVER_CONFIG.default_gamemode
                        )
                        .as_str(),
                    );
                    crate::types::player::Gamemode::Survival
                }
            },

            health: 20.0,
            hunger: 20,
            saturation: 5.0,
            dead: false,

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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct WorldSave {
    pub timestamp: i64,
}

impl Default for WorldSave {
    fn default() -> Self {
        Self { timestamp: 0 }
    }
}

pub fn ensure_save_folders() -> anyhow::Result<()> {
    std::fs::create_dir_all(crate::config::SERVER_CONFIG.world_name.clone())?;

    std::fs::create_dir_all(format!(
        "{}/players",
        crate::config::SERVER_CONFIG.world_name.clone()
    ))?;

    std::fs::create_dir_all(format!(
        "{}/regions",
        crate::config::SERVER_CONFIG.world_name.clone()
    ))?;

    Ok(())
}

pub fn exists(path: &str) -> bool {
    std::path::Path::new(
        format!(
            "{}/{}",
            crate::config::SERVER_CONFIG.world_name.clone(),
            path
        )
        .as_str(),
    )
    .exists()
}

pub async fn save() -> anyhow::Result<()> {
    ensure_save_folders()?;
    crate::log::log(LogLevel::Info, "Saving...\n");

    let start = std::time::Instant::now();

    let mut handles = Vec::new();

    for player in crate::server::state::play::PLAYERS.read().await.values() {
        let player = player.clone();

        handles.push(tokio::spawn(async move {
            let player = player.lock().await;

            if let Err(e) = save_player(&player).await {
                crate::log::log(
                    LogLevel::Error,
                    format!("Failed to save player {}: {}", player.username, e).as_str(),
                );
            }
        }))
    }

    for region in crate::world::REGIONS.lock().await.values() {
        let region = region.clone();

        handles.push(tokio::spawn(async move {
            let region = region.lock().await;
            let _ = region.save().await;
        }))
    }

    for handle in handles {
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
    }

    save_world_data(&WorldSave {
        timestamp: crate::server::tick::TIMESTAMP.load(Ordering::Relaxed),
    })?;

    let duration = start.elapsed();

    crate::log::log(
        LogLevel::Info,
        format!("Save complete in {:.2?}\n", duration).as_str(),
    );

    Ok(())
}

pub async fn save_player(player: &crate::types::player::Player) -> anyhow::Result<()> {
    let inventory = player.inventory.iter().map(|slot| slot.clone()).collect();

    let player_save = PlayerSave {
        uuid: player.uuid.clone(),
        inventory,

        is_op: player.is_op,
        gamemode: player.gamemode,

        health: player.health,
        hunger: player.hunger,
        saturation: player.saturation,
        dead: player.dead,

        x: player.x,
        y: player.y,
        z: player.z,

        vx: player.vx,
        vy: player.vy,
        vz: player.vz,

        yaw: player.yaw,
        pitch: player.pitch,
    };

    let player_json = serde_json::to_string_pretty(&player_save)?;
    let player_path = format!(
        "{}/players/{}.json",
        crate::config::SERVER_CONFIG.world_name.clone(),
        player.uuid
    );

    std::fs::write(player_path, player_json)?;

    Ok(())
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

    let Ok(player_json) = std::fs::read_to_string(player_path) else {
        log::log(
            LogLevel::Warn,
            format!("Failed to read player save for UUID: {}", uuid).as_str(),
        );

        return None;
    };

    let Ok(player_save) = serde_json::from_str(&player_json) else {
        log::log(
            LogLevel::Warn,
            format!("Failed to parse player save for UUID: {}", uuid).as_str(),
        );

        return None;
    };

    Some(player_save)
}

pub fn save_world_data(world_save: &WorldSave) -> anyhow::Result<()> {
    let world_path = format!(
        "{}/world.json",
        crate::config::SERVER_CONFIG.world_name.clone()
    );

    let world_json = serde_json::to_string_pretty(world_save)?;
    std::fs::write(world_path, world_json)?;

    Ok(())
}

pub fn load_world_data() -> anyhow::Result<WorldSave> {
    let world_path = format!(
        "{}/world.json",
        crate::config::SERVER_CONFIG.world_name.clone()
    );

    if !std::path::Path::new(&world_path).exists() {
        return Ok(WorldSave::default());
    }

    let world_json = std::fs::read_to_string(world_path)?;
    let save = serde_json::from_str(&world_json);

    Ok(save.unwrap_or_default())
}
