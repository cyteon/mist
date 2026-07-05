use std::time::Duration;
use tokio::{task, try_join};

use crate::log::LogLevel;
use crate::server::save::{self, save, save_player};

pub async fn run() -> anyhow::Result<()> {
    crate::log::log(
        LogLevel::Info,
        format!(
            "Starting server on {}:{}",
            crate::config::SERVER_CONFIG.host,
            crate::config::SERVER_CONFIG.port
        )
        .as_str(),
    );

    crate::server::save::ensure_save_folders();

    if !save::exists("regions/0_0.mist_region") {
        crate::world::worldgen::initial_gen().await;
    }

    // server setup stuff goes here before listener activates

    let listener_task = task::spawn(crate::server::listener::start_listener());
    let tick_task = task::spawn(crate::server::tick::start_tick_loop());

    let _ = try_join!(listener_task, tick_task)?;

    Ok(())
}

pub async fn stop() {
    crate::log::log(LogLevel::Info, "Stopping server...\n");

    let players = crate::server::state::play::PLAYERS
        .write()
        .await
        .drain()
        .map(|(_, p)| p)
        .collect::<Vec<_>>();

    crate::server::conn::PLAYER_SOCKET_MAP.write().await.clear();

    for player in players {
        match tokio::time::timeout(Duration::from_millis(500), player.lock()).await {
            Ok(player) => {
                save_player(&player).await;
            }

            Err(_) => {
                crate::log::log(
                    LogLevel::Warn,
                    format!(
                        "Timeout while saving player {}",
                        player.lock().await.username
                    )
                    .as_str(),
                );
            }
        }
    }

    crate::log::log(LogLevel::Info, "Saving world...\n");

    save().await;
}
