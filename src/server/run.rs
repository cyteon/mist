use std::time::Duration;
use tokio::{task, try_join};

use crate::log::LogLevel;
use crate::net::packets::clientbound::disconnect::send_disconnect_play;
use crate::server::conn::PLAYER_SOCKET_MAP;
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

    save::ensure_save_folders()?;

    if !save::exists("regions/0_0.mist_region") {
        crate::world::worldgen::initial_gen().await?;
    }

    // server setup stuff goes here before listener activates

    let listener_task = task::spawn(crate::server::listener::start_listener());
    let tick_task = task::spawn(crate::server::tick::start_tick_loop());

    let _ = try_join!(listener_task, tick_task)?;

    Ok(())
}

pub async fn stop() -> anyhow::Result<()> {
    crate::log::log(LogLevel::Info, "Stopping server...\n");

    let players = crate::server::state::play::PLAYERS
        .write()
        .await
        .drain()
        .map(|(_, p)| p)
        .collect::<Vec<_>>();

    let mut buffer = Vec::new();
    send_disconnect_play(&mut buffer, "Server is stopping").await?;

    for tx in PLAYER_SOCKET_MAP.write().await.values() {
        let _ = tx.send(buffer.clone());
    }

    PLAYER_SOCKET_MAP.write().await.clear();

    for player in players {
        match tokio::time::timeout(Duration::from_millis(500), player.lock()).await {
            Ok(player) => {
                if let Err(e) = save_player(&player).await {
                    crate::log::log(
                        LogLevel::Error,
                        format!("Failed to save player {}: {}", player.username, e).as_str(),
                    );
                }
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
    save().await?;

    Ok(())
}
