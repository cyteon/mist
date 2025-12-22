use std::{collections::HashMap, time::Duration};

use fancy_log::{LogLevel, log};
use once_cell::sync::Lazy;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock, time::{self, timeout}};

use crate::{
    net::{packet::{
        ClientPacket, ProtocolState, read_packet
    }, packets::{clientbound::{keep_alive::send_keep_alive, sync_player_position::send_sync_player_position}, serverbound::confirm_teleportation::read_confirm_teleportation}}, 
    
    server::{conn::PLAYER_SOCKET_MAP, encryption::EncryptedStream},
    types::player::Player
};

use std::sync::Arc;
use tokio::sync::Mutex;

pub static PLAYERS: Lazy<RwLock<HashMap<String, Arc<Mutex<Player>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn play(socket: EncryptedStream<TcpStream>, mut player: Player) -> anyhow::Result<()> {
    log(LogLevel::Debug, format!("{} has entered the play state", player.name).as_str());

    let socket = Arc::new(Mutex::new(socket));
    let player = Arc::new(Mutex::new(player));

    let keep_alive_future ={
        let socket = Arc::clone(&socket);

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(15));

            loop {
                interval.tick().await;

                let mut socket = socket.lock().await;
                send_keep_alive(&mut *socket).await.unwrap();
            }
        })
    };

    send_sync_player_position(&mut *socket.lock().await, &*player.lock().await).await?;

    log(
        LogLevel::Debug, 
        format!("Sent initial player position to {}", player.lock().await.name).as_str()
    );

    PLAYER_SOCKET_MAP.write().await.insert(
        player.lock().await.name.clone(),
        Arc::clone(&socket)
    );

    PLAYERS.write().await.insert(
        player.lock().await.name.clone(),
        Arc::clone(&player)
    );

    loop {
        let mut socket_guard = socket.lock().await;

        match timeout(Duration::from_secs(20), read_packet(&mut *socket_guard, &ProtocolState::Play)).await {
            Ok(Ok(Some(packet))) => {
                match packet {
                    ClientPacket::ConfirmTeleprortion(mut cursor) => {
                        read_confirm_teleportation(&mut cursor, &mut *player.lock().await).await?;
                    }

                    _ => { }
                }
            },

            Ok(Ok(None)) => { },

            Err(e) => { 
                log(
                    LogLevel::Error, 
                    format!("{} has timed out during play state: {}", player.lock().await.name, e).as_str()
                );

                socket_guard.shutdown().await?;
                keep_alive_future.abort();
                break; 
            }
                
            Ok(Err(e)) => {
                log(
                    LogLevel::Error, 
                    format!("Error while reading packet from {} during play state: {}", player.lock().await.name, e).as_str()
                );

                socket_guard.shutdown().await?;
                keep_alive_future.abort();
                break; 
            }
        }
    }

    Ok(())
}