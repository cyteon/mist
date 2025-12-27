use std::{collections::HashMap, time::Duration};

use fancy_log::LogLevel;
use once_cell::sync::Lazy;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::{RwLock, Mutex}, time::{self, timeout}};
use std::sync::Arc;
use futures::future::join_all;

use crate::{
    net::{
        packet::{
            ClientPacket, ProtocolState, read_packet
        }, 

        packets::{
            clientbound::{
                chunk_data_with_light::send_chunk_data_with_light, 
                game_event::send_game_event,
                keep_alive::send_keep_alive,
                set_center_chunk::send_set_center_chunk,
                sync_player_position::send_sync_player_position
            },

            serverbound::{
                confirm_teleportation::read_confirm_teleportation, 
                player_action::read_player_action, 
                use_item_on::read_use_item_on
            }
        }
    }, 
    
    server::{conn::PLAYER_SOCKET_MAP, encryption::EncryptedStream},
    types::player::Player
};

pub static PLAYERS: Lazy<RwLock<HashMap<String, Arc<Mutex<Player>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn play(socket: EncryptedStream<TcpStream>, mut player: Player) -> anyhow::Result<()> {
    crate::log::log(LogLevel::Debug, format!("{} has entered the play state", player.name).as_str());

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

    crate::log::log(
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

    send_game_event(&mut *socket.lock().await, 13, 0.0).await?;
    send_set_center_chunk(&mut *socket.lock().await, 0, 0).await?;

    crate::log::log(
        LogLevel::Debug, 
        format!("Sent center chunk and is now sending chunks to {}", player.lock().await.name).as_str()
    );

    let chunk_sender_task = {
        let socket = Arc::clone(&socket);
        let player_name = player.lock().await.name.clone();
        let view_distance = crate::config::SERVER_CONFIG.view_distance as i32;
        let chunk_loading_width = view_distance * 2 + 7;

        let regions_lock = crate::world::worldgen::REGIONS.lock().await;
        
        let mut chunks_to_send = Vec::new();
        for cx in -chunk_loading_width/2..=chunk_loading_width/2 {
            for cz in -chunk_loading_width/2..=chunk_loading_width/2 {
                if let Some(region) = regions_lock.get(&(cx >> 5, cz >> 5)) {
                    if let Some(chunk) = region.chunks.iter().find(|chunk| chunk.x == cx && chunk.z == cz) {
                        chunks_to_send.push(chunk.clone());
                    }
                }
            }
        }
        drop(regions_lock);

        // sort so chunk loading starts at 0,0
        chunks_to_send.sort_by_key(|chunk| {
            chunk.x * chunk.x + chunk.z * chunk.z
        });

        tokio::spawn(async move {
            let mut encoding_tasks = Vec::new();
            for chunk in chunks_to_send {
                encoding_tasks.push(tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    send_chunk_data_with_light(&mut buffer, &chunk).await?;
                    Ok::<Vec<u8>, anyhow::Error>(buffer)
                }));
            }

            let results = join_all(encoding_tasks).await;
            for result in results {
                if let Ok(Ok(packet)) = result {
                    let mut socket = socket.lock().await;
                    if socket.write_all(&packet).await.is_err() {
                        break;
                    }
                    let _ = socket.flush().await;
                }
            }

            crate::log::log(
                LogLevel::Debug, 
                format!("Finished sending chunks to {}", player_name).as_str()
            );
        })
    };

    loop {
        let mut socket_guard = socket.lock().await;

        match timeout(Duration::from_secs(20), read_packet(&mut *socket_guard, &ProtocolState::Play)).await {
            Ok(Ok(Some(packet))) => {
                match packet {
                    ClientPacket::ConfirmTeleprortion(mut cursor) => {
                        read_confirm_teleportation(&mut cursor, &mut *player.lock().await).await?;
                    }
                    
                    ClientPacket::PlayerAction(mut cursor) => {
                        read_player_action(&mut cursor).await?;
                    }

                    ClientPacket::UseItemOn(mut cursor) => {
                        read_use_item_on(&mut cursor, &mut *player.lock().await).await?;
                    }

                    _ => { }
                }
            },

            Ok(Ok(None)) => { },

            Err(e) => { 
                crate::log::log(
                    LogLevel::Error, 
                    format!("{} has timed out during play state: {}", player.lock().await.name, e).as_str()
                );

                socket_guard.shutdown().await?;
                keep_alive_future.abort();
                chunk_sender_task.abort();
                break; 
            }
                
            Ok(Err(e)) => {
                crate::log::log(
                    LogLevel::Error, 
                    format!("Error while reading packet from {} during play state: {}", player.lock().await.name, e).as_str()
                );

                socket_guard.shutdown().await?;
                keep_alive_future.abort();
                chunk_sender_task.abort();
                break; 
            }
        }
    }

    Ok(())
}