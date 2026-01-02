use std::{collections::HashMap, time::Duration};

use fancy_log::LogLevel;
use once_cell::sync::Lazy;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::{Mutex, RwLock}, time::{self, timeout}};
use std::sync::Arc;

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
                player_chat_message::send_player_chat_message,
                player_info_remove::send_player_info_remove,
                player_info_update::{PlayerAction, send_player_info_update},
                set_center_chunk::send_set_center_chunk,
                sync_player_position::send_sync_player_position
            },

            serverbound::{
                chat_message::read_chat_message,
                confirm_teleportation::read_confirm_teleportation,
                player_action::read_player_action, player_input::read_player_input,
                set_player_position_and_rotation::read_set_player_position_and_rotation,
                set_player_rotation::read_set_player_rotation,
                use_item_on::read_use_item_on
            }
        }
    }, 
    
    server::{conn::PLAYER_SOCKET_MAP, encryption::EncryptedStream},
    types::player::Player, world::worldgen::get_region
};

pub static PLAYERS: Lazy<RwLock<HashMap<String, Arc<Mutex<Player>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn play(socket: EncryptedStream<TcpStream>, mut player: Player) -> anyhow::Result<()> {
    crate::log::log(LogLevel::Debug, format!("{} has entered the play state", player.username).as_str());

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
        format!("Sent initial player position to {}", player.lock().await.username).as_str()
    );

    PLAYER_SOCKET_MAP.write().await.insert(
        player.lock().await.uuid.clone(),
        Arc::clone(&socket)
    );

    PLAYERS.write().await.insert(
        player.lock().await.uuid.clone(),
        Arc::clone(&player)
    );

    crate::log::log(
        LogLevel::Debug, 
        format!("Added {} to player list", player.lock().await.username).as_str()
    );

    {
        let player_guard = player.lock().await;
        let player_clone = player_guard.clone();
        drop(player_guard);

        let players_guard = PLAYERS.read().await;
        let players = players_guard.clone();
        drop(players_guard);
        
        let mut other_players_owned = Vec::new();
        for p in players.values() {
            let p_guard = p.lock().await;

            if p_guard.uuid != player_clone.uuid {
                other_players_owned.push(p_guard.clone());
            }

            drop(p_guard);
        }

        if !other_players_owned.is_empty() {
            let mut socket_guard = socket.lock().await;

            send_player_info_update(
                &mut *socket_guard, 
                other_players_owned.iter().collect(),
                vec![PlayerAction::AddPlayer, PlayerAction::UpdateListed(true)]
            ).await?;            
        }
    }

    for player_socket in PLAYER_SOCKET_MAP.read().await.values() {
        let player_guard = player.lock().await;
        let player_clone = player_guard.clone();
        drop(player_guard);

        let mut socket_guard: tokio::sync::MutexGuard<'_, EncryptedStream<TcpStream>> = player_socket.lock().await;

        send_player_info_update(
            &mut *socket_guard, 
            vec![&player_clone],
            vec![PlayerAction::AddPlayer, PlayerAction::UpdateListed(true)]
        ).await?;
    }

    crate::log::log(
        LogLevel::Debug, 
        format!("Sent player info updates for {}", player.lock().await.username).as_str()
    );

    send_game_event(&mut *socket.lock().await, 13, 0.0).await?;
    send_set_center_chunk(&mut *socket.lock().await, 0, 0).await?;

    crate::log::log(
        LogLevel::Debug, 
        format!("Sent center chunk and is now sending chunks to {}", player.lock().await.username).as_str()
    );

    let chunk_sender_task = {
        let socket = Arc::clone(&socket);
        let player_name = player.lock().await.username.clone();
        let view_distance = crate::config::SERVER_CONFIG.view_distance as i32;
        let chunk_loading_width = view_distance * 2 + 7;
        
        let mut chunks_to_send = Vec::new();
        for cx in -chunk_loading_width/2..=chunk_loading_width/2 {
            for cz in -chunk_loading_width/2..=chunk_loading_width/2 {
                if let Some(region) = get_region(cx >> 5, cz >> 5).await {
                    if let Some(chunk) = region.chunks.iter().find(|chunk| chunk.x == cx && chunk.z == cz) {
                        chunks_to_send.push(chunk.clone());
                    }
                }
            }
        }

        // sort so chunk loading starts at 0,0
        chunks_to_send.sort_by_key(|chunk| {
            chunk.x * chunk.x + chunk.z * chunk.z
        });

        tokio::spawn(async move {            
            for batch in chunks_to_send.chunks(8) {
                let mut encoding_tasks = Vec::new();
                for chunk in batch {
                    let chunk = chunk.clone();
                    encoding_tasks.push(tokio::spawn(async move {
                        let mut buffer = Vec::new();
                        send_chunk_data_with_light(&mut buffer, &chunk).await?;
                        Ok::<Vec<u8>, anyhow::Error>(buffer)
                    }));
                }

                let results = futures::future::join_all(encoding_tasks).await;
                
                {
                    let mut socket = socket.lock().await;
                    for result in results {
                        if let Ok(Ok(packet)) = result {
                            if socket.write_all(&packet).await.is_err() {
                                return;
                            }
                        }
                    }

                    if socket.flush().await.is_err() {
                        return;
                    }
                }
                
                tokio::task::yield_now().await;
            }

            crate::log::log(
                LogLevel::Debug, 
                format!("Finished sending chunks to {}", player_name).as_str()
            );
        })
    };

    loop {
        let mut socket_guard = socket.lock().await;

        match timeout(Duration::from_secs(30), read_packet(&mut *socket_guard, &ProtocolState::Play)).await {
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

                    ClientPacket::ChatMessage(mut cursor) => {
                        let message = read_chat_message(&mut cursor).await?;

                        let player_guard = player.lock().await;
                        let player_clone = player_guard.clone();
                        drop(player_guard);

                        crate::log::log(
                            LogLevel::Info, 
                            format!("<{}> {}", player_clone.username, message.content).as_str()
                        );

                        drop(socket_guard);
                        
                        for player in PLAYERS.read().await.values() {
                            let mut target_player_guard = player.lock().await;

                            let target_player_socket = PLAYER_SOCKET_MAP.read().await;
                            let target_player_socket = target_player_socket.get(&target_player_guard.uuid).unwrap();
                            let mut socket_guard = target_player_socket.lock().await;

                            send_player_chat_message(
                                &mut *socket_guard,
                                &player_clone,
                                &mut *target_player_guard,
                                &message
                            ).await?;
                            
                            drop(socket_guard);
                        }
                        
                        continue;
                    }

                    ClientPacket::SetPlayerPositionAndRotation(mut cursor) => {
                        read_set_player_position_and_rotation(&mut cursor, &mut *player.lock().await).await?;
                    }

                    ClientPacket::PlayerInput(mut cursor) => {
                        read_player_input(&mut cursor, &mut *player.lock().await).await?;
                    }

                    ClientPacket::SetPlayerRotation(mut cursor) => {
                        read_set_player_rotation(&mut cursor, &mut *player.lock().await).await?;
                    }

                    _ => { }
                }
            },

            Ok(Ok(None)) => { },

            Err(e) => { 
                crate::log::log(
                    LogLevel::Error, 
                    format!("{} has timed out during play state: {}", player.lock().await.username, e).as_str()
                );

                PLAYER_SOCKET_MAP.write().await.remove(&player.lock().await.uuid);
                PLAYERS.write().await.remove(&player.lock().await.uuid);

                socket_guard.shutdown().await?;
                keep_alive_future.abort();
                chunk_sender_task.abort();

                for other_player in PLAYER_SOCKET_MAP.read().await.values().into_iter() {
                    let socket_lock = &mut *other_player.lock().await;
                    send_player_info_remove(socket_lock, vec![&player.lock().await.uuid]).await?;
                }

                break; 
            }
                
            Ok(Err(e)) => {
                crate::log::log(
                    LogLevel::Error, 
                    format!("Error while reading packet from {} during play state: {}", player.lock().await.username, e).as_str()
                );

                PLAYER_SOCKET_MAP.write().await.remove(&player.lock().await.uuid);
                PLAYERS.write().await.remove(&player.lock().await.uuid);

                socket_guard.shutdown().await?;
                keep_alive_future.abort();
                chunk_sender_task.abort();

                for other_player in PLAYER_SOCKET_MAP.read().await.values().into_iter() {
                    let socket_lock = &mut *other_player.lock().await;
                    send_player_info_remove(socket_lock, vec![&player.lock().await.uuid]).await?;
                }

                break; 
            }
        }
    }

    Ok(())
}