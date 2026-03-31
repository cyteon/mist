use fancy_log::LogLevel;
use once_cell::sync::Lazy;
use std::{collections::HashMap, time::Duration, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{Mutex, RwLock, mpsc},
    time::{self, timeout}
};

use crate::{
    net::{
        packet::{
            ClientPacket, ProtocolState, read_packet, encode_packet
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
                sync_player_position::send_sync_player_position,
                commands::send_commands
            },

            serverbound::{
                chat_message::read_chat_message,
                confirm_teleportation::read_confirm_teleportation,
                player_action::read_player_action,
                player_input::read_player_input,
                set_player_position_and_rotation::read_set_player_position_and_rotation,
                set_player_rotation::read_set_player_rotation,
                use_item_on::read_use_item_on,
                chat_command::read_chat_command
            }
        }
    }, 
    
    server::{conn::PLAYER_SOCKET_MAP, encryption::EncryptedStream, commands::handle_command},
    types::player::Player, world::get_region
};

pub static PLAYERS: Lazy<RwLock<HashMap<String, Arc<Mutex<Player>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn play(socket: EncryptedStream<TcpStream>, player: Player) -> anyhow::Result<()> {
    crate::log::log(LogLevel::Debug, format!("{} has entered the play state", player.username).as_str());
    crate::log::log(LogLevel::Info, format!("{} ({}) joined the server", player.username, player.uuid).as_str());

    let uuid = player.uuid.clone();
    let username = player.username.clone();
    let mut player = player;

    let (mut read, write) = socket.into_split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

    let writer_future = tokio::spawn(async move {
        let mut write = write;

        while let Some(buffer) = rx.recv().await {
            let packet = encode_packet(&buffer);

            if let Err(_) = write.write_all(&packet).await {
                break;
            }

            while let Ok(buffer) = rx.try_recv() {
                let packet = encode_packet(&buffer);
                
                if let Err(_) = write.write_all(&packet).await {
                    break;
                }
            }

            if let Err(_) = write.flush().await {
                break;
            }
        }

        let _ = write.shutdown().await;
    });

    let keep_alive_future = {
        let tx = tx.clone();
        let username = username.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(15));

            loop {
                interval.tick().await;

                let mut buffer = Vec::new();
                if send_keep_alive(&mut buffer).await.is_ok() {
                    let _ = tx.send(buffer);
                    
                    crate::log::log(
                        LogLevel::Debug, 
                        format!("Sent keep alive to {}", username).as_str()
                    );
                }
            }
        })
    };
    
    if player.x == 0.0 && player.y == 0.0 && player.z == 0.0 {
        let surface_y = 
            get_region(player.x as i32 >> 9, player.z as i32 >> 9).await.lock().await
            .get_chunk(player.x as i32 >> 4, player.z as i32 >> 4).unwrap()
            .get_surface_y((player.x as i32 & 15) as u8, (player.z as i32 & 15) as u8);
        
        player.y = surface_y as f64 + 1.0;
    }

    let mut buffer = Vec::new();
    send_sync_player_position(&mut buffer, &player).await?;
    let _ = tx.send(buffer);

    crate::log::log(
        LogLevel::Debug, 
        format!("Sent initial player position to {}", username).as_str()
    );

    let mut buffer = Vec::new();
    send_commands(&mut buffer).await?;
    let _ = tx.send(buffer);

    let player = Arc::new(Mutex::new(player));

    PLAYER_SOCKET_MAP.write().await.insert(
        player.lock().await.uuid.clone(),
        tx.clone()
    );

    PLAYERS.write().await.insert(
        player.lock().await.uuid.clone(),
        Arc::clone(&player)
    );

    let mut player_guard = player.lock().await;
    player_guard.sync_player_inventory().await?;
    drop(player_guard);

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
            let mut buffer = Vec::new();

            send_player_info_update(
                &mut buffer,
                other_players_owned.iter().collect(),
                vec![PlayerAction::AddPlayer, PlayerAction::UpdateListed(true)]
            ).await?;
            
            let _ = tx.send(buffer);
        }
    }

    for player_tx in PLAYER_SOCKET_MAP.read().await.values() {
        let player_guard = player.lock().await;
        let player_clone = player_guard.clone();
        drop(player_guard);

        let mut buffer = Vec::new();

        send_player_info_update(
            &mut buffer,
            vec![&player_clone],
            vec![PlayerAction::AddPlayer, PlayerAction::UpdateListed(true)]
        ).await?;

        let _ = player_tx.send(buffer);
    }

    crate::log::log(
        LogLevel::Debug, 
        format!("Sent player info updates for {}", player.lock().await.username).as_str()
    );

    let mut buffer = Vec::new();
    send_game_event(&mut buffer, 13, 0.0).await?;
    let _ = tx.send(buffer);

    let mut buffer = Vec::new();
    send_set_center_chunk(&mut buffer, 0, 0).await?;
    let _ = tx.send(buffer);

    crate::log::log(
        LogLevel::Debug, 
        format!("Sent center chunk and is now sending chunks to {}", player.lock().await.username).as_str()
    );

    let chunk_sender_task = {
        let tx = tx.clone();
        let player_name = player.lock().await.username.clone();
        let view_distance = crate::config::SERVER_CONFIG.view_distance as i32;
        let chunk_loading_width = view_distance * 2 + 7;
        let radius = chunk_loading_width / 2;

        let mut by_region: HashMap<(i32, i32), Vec<(i32, i32)>> = HashMap::new();
        for cx in -radius..=radius {
            for cz in -radius..=radius {
                by_region.entry((cx >> 5, cz >> 5)).or_default().push((cx, cz));
            }
        }
        
        let mut chunks_to_send = Vec::new();
        for ((rx, rz), coords) in by_region {
            let region = get_region(rx, rz).await.lock().await.clone();

            for (cx, cz) in coords {
                let chunk = region.get_chunk(cx, cz).unwrap();
                chunks_to_send.push(chunk.clone());
            }
        }

        // sort so chunk loading starts at 0,0
        chunks_to_send.sort_by_key(|chunk| {
            chunk.x * chunk.x + chunk.z * chunk.z
        });

        tokio::spawn(async move {
            crate::log::log(
                LogLevel::Debug, 
                format!("Making chunk packets for {} to send", player_name).as_str()
            );

            let tasks: Vec<_> = chunks_to_send.into_iter().map(|chunk| {
                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    send_chunk_data_with_light(&mut buffer, &chunk).await?;

                    Ok::<Vec<u8>, anyhow::Error>(buffer)
                })
            }).collect();

            let results: Vec<_> = futures::future::join_all(tasks).await;

            crate::log::log(
                LogLevel::Debug, 
                format!("Finished making chunk packets for {}, now sending", player_name).as_str()
            );

            for result in results {
                if let Ok(Ok(pkt)) = result {
                    let _ = tx.send(pkt);
                }
            }

            crate::log::log(
                LogLevel::Debug, 
                format!("Finished sending chunks to {}", player_name).as_str()
            );

            let players_locked = PLAYERS.read().await;
            let player_lock = players_locked.get(&player.lock().await.uuid).unwrap().clone();
            let mut player = player_lock.lock().await;
            player.chunks_loaded = true;
        })
    };

    loop {
        match timeout(Duration::from_secs(30), read_packet(&mut read, &ProtocolState::Play, true)).await {
            Ok(Ok(Some(packet))) => {
                match packet {
                    ClientPacket::ConfirmTeleprortion(mut cursor) => {
                        let players_locked = PLAYERS.read().await;
                        let mut player = players_locked.get(&uuid).unwrap().lock().await;
                        read_confirm_teleportation(&mut cursor, &mut player).await?;
                    }
                    
                    ClientPacket::PlayerAction(mut cursor) => {
                        read_player_action(&mut cursor).await?;
                    }

                    ClientPacket::UseItemOn(mut cursor) => {
                        let players_locked = PLAYERS.read().await;
                        let mut player = players_locked.get(&uuid).unwrap().lock().await;
                        read_use_item_on(&mut cursor, &mut player).await?;
                    }

                    ClientPacket::ChatCommand(mut cursor) => {
                        let command = read_chat_command(&mut cursor).await?;

                        crate::log::log(
                            LogLevel::Info, 
                            format!("{} issued command /{}", username, command).as_str()
                        );

                        let players_locked = PLAYERS.read().await;
                        let mut player = players_locked.get(&uuid).unwrap().lock().await;
                        handle_command(command, &mut player).await?;
                    }

                    ClientPacket::ChatMessage(mut cursor) => {
                        let message = read_chat_message(&mut cursor).await?;

                        let players_locked = PLAYERS.read().await;
                        let player_clone = players_locked.get(&uuid).unwrap().lock().await.clone();
                        drop(players_locked);

                        crate::log::log(
                            LogLevel::Info, 
                            format!("<{}> {}", player_clone.username, message.content).as_str()
                        );
                        
                        for player in PLAYERS.read().await.values() {
                            let mut target_player_guard = player.lock().await;

                            if target_player_guard.uuid == uuid {
                                let mut buffer = Vec::new();

                                send_player_chat_message(
                                    &mut buffer,
                                    &player_clone,
                                    &mut *target_player_guard,
                                    &message
                                ).await?;

                                let _ = tx.send(buffer);
                            } else {
                                let player_sockets = PLAYER_SOCKET_MAP.read().await;
                                let target_player_tx = player_sockets.get(&target_player_guard.uuid).unwrap();
                                
                                let mut buffer = Vec::new();

                                send_player_chat_message(
                                    &mut buffer,
                                    &player_clone,
                                    &mut *target_player_guard,
                                    &message
                                ).await?;
                                
                                let _ = target_player_tx.send(buffer);
                            }  
                        }
                    }

                    ClientPacket::SetPlayerPositionAndRotation(mut cursor) => {
                        let players_locked = PLAYERS.read().await;
                        let mut player = players_locked.get(&uuid).unwrap().lock().await;

                        read_set_player_position_and_rotation(&mut cursor, &mut player).await?;
                    }

                    ClientPacket::PlayerInput(mut cursor) => {
                        let players_locked = PLAYERS.read().await;
                        let mut player = players_locked.get(&uuid).unwrap().lock().await;

                        read_player_input(&mut cursor, &mut player).await?;
                    }

                    ClientPacket::SetPlayerRotation(mut cursor) => {
                        let players_locked = PLAYERS.read().await;
                        let mut player = players_locked.get(&uuid).unwrap().lock().await;

                        read_set_player_rotation(&mut cursor, &mut player).await?;
                    }

                    _ => { }
                }
            },

            Ok(Ok(None)) => { },

            Err(e) => { 
                crate::log::log(
                    LogLevel::Error, 
                    format!("{} has timed out during play state: {}", username, e).as_str()
                );

                break; 
            }
                
            Ok(Err(e)) => {
                crate::log::log(
                    LogLevel::Error, 
                    format!("Error while reading packet from {} during play state: {}", username, e).as_str()
                );

                break; 
            }
        }
    }

    crate::log::log(
        LogLevel::Info, 
        format!("{} ({}) left the server", username, uuid).as_str()
    );

    PLAYER_SOCKET_MAP.write().await.remove(&uuid);

    drop(tx);

    writer_future.abort();
    keep_alive_future.abort();
    chunk_sender_task.abort();

    if PLAYERS.read().await.len() == 1 {
        crate::log::log(
            LogLevel::Debug, 
            "No players online, saving and clearing regions from memory"
        );

        crate::server::save::save().await;
        crate::world::REGIONS.lock().await.clear(); // unnecesary having all regions loaded in while nobody is playing
    } else {
        let players_locked = PLAYERS.read().await;
        let player_lock = players_locked.get(&uuid).unwrap().clone();
        crate::server::save::save_player(&player_lock.lock().await.clone()).await;

        for other_tx in PLAYER_SOCKET_MAP.read().await.values().into_iter() {
            let mut buffer = Vec::new();
            send_player_info_remove(&mut buffer, vec![&uuid]).await?;

            let _ = other_tx.send(buffer);
        }
    }

    PLAYERS.write().await.remove(&uuid);

    Ok(())
}