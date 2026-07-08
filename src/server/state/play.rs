use crate::{
    log::LogLevel,
    net::packets::clientbound::block_action::send_block_action,
    server::save,
    types::{
        block_entities::BlockEntityData,
        colors,
        entity::ENTITIES,
        player::{PLAYER_POSITIONS, WindowType, broadcast_packet, broadcast_system_message},
    },
    world::REGIONS,
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{Mutex, RwLock, mpsc},
    time::{self, timeout},
};

use crate::{
    net::{
        packet::{ClientPacket, ProtocolState, encode_packet, read_packet},
        packets::{
            clientbound::{
                commands::send_commands,
                game_event::send_game_event,
                keep_alive::send_keep_alive,
                level_chunk_with_light::send_level_chunk_with_light,
                player_chat_message::send_player_chat_message,
                player_info_remove::send_player_info_remove,
                player_info_update::{PlayerAction, send_player_info_update},
                set_center_chunk::send_set_center_chunk,
                set_ticking_state::send_set_ticking_state,
            },
            serverbound::{
                chat_command::read_chat_command, chat_message::read_chat_message,
                client_status::read_client_status,
                confirm_teleportation::read_confirm_teleportation,
                container_click::read_container_click, container_close::read_container_close,
                pick_item_from_block::read_pick_item_from_block,
                player_abilities::read_player_abilities, player_action::read_player_action,
                player_input::read_player_input, set_carried_item::read_set_carried_item,
                set_creative_mode_slot::read_set_creative_mode_slot,
                set_player_position::read_set_player_position,
                set_player_position_and_rotation::read_set_player_position_and_rotation,
                set_player_rotation::read_set_player_rotation, swing_arm::read_swing_arm,
                use_item::read_use_item, use_item_on::read_use_item_on,
            },
        },
    },
    server::{
        commands::{CommandInvoker, handle_command},
        conn::PLAYER_SOCKET_MAP,
        encryption::EncryptedStream,
    },
    types::player::Player,
    world::{get_chunk, get_region},
};

pub static PLAYERS: Lazy<RwLock<HashMap<String, Arc<Mutex<Player>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub static NAME_TO_UUID: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn send_chunks_to_player(
    tx: mpsc::UnboundedSender<Vec<u8>>,
    player: Arc<Mutex<Player>>,
) -> anyhow::Result<()> {
    let view_distance = crate::config::SERVER_CONFIG.view_distance as i32;
    let chunk_loading_width = view_distance * 2 + 7;
    let radius = chunk_loading_width / 2;

    let cx = player.lock().await.x as i32 >> 4;
    let cz = player.lock().await.z as i32 >> 4;

    let mut buffer = Vec::new();
    send_game_event(&mut buffer, 13, 0.0).await?;
    let _ = tx.send(buffer);

    let mut buffer = Vec::new();
    send_set_center_chunk(&mut buffer, cx, cz).await?;
    let _ = tx.send(buffer);

    let mut by_region: HashMap<(i32, i32), Vec<(i32, i32)>> = HashMap::new();
    for cx in -radius..=radius {
        for cz in -radius..=radius {
            by_region
                .entry((cx >> 5, cz >> 5))
                .or_default()
                .push((cx, cz));
        }
    }

    let mut chunks_to_send = Vec::new();
    for ((rx, rz), coords) in by_region {
        let region_arc = get_region(rx, rz).await;

        for (cx, cz) in coords {
            let chunk = get_chunk(&region_arc, cx, cz).await;
            chunks_to_send.push(chunk);
        }
    }

    // sort so chunk loading starts at 0,0
    chunks_to_send.sort_by_key(|chunk| chunk.x * chunk.x + chunk.z * chunk.z);

    let tasks: Vec<_> = chunks_to_send
        .into_iter()
        .map(|chunk| {
            tokio::spawn(async move {
                let mut buffer = Vec::new();
                send_level_chunk_with_light(&mut buffer, &chunk).await?;

                Ok::<Vec<u8>, anyhow::Error>(buffer)
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(tasks).await;

    for result in results {
        if let Ok(Ok(pkt)) = result {
            let _ = tx.send(pkt);
        }
    }

    let uuid = player.lock().await.uuid.clone();
    let players_locked = PLAYERS.read().await;
    if let Some(player_lock) = players_locked.get(&uuid) {
        let mut player = player_lock.lock().await;
        player.chunks_loaded = true;
    }

    Ok(())
}

pub async fn play(socket: EncryptedStream<TcpStream>, player: Player) -> anyhow::Result<()> {
    crate::log::log(
        LogLevel::Debug,
        format!("{} has entered the play state", player.username).as_str(),
    );
    crate::log::log(
        LogLevel::Info,
        format!("{} ({}) joined the server", player.username, player.uuid).as_str(),
    );

    let uuid = player.uuid.clone();
    let username = player.username.clone();
    let mut player = player;

    let (mut read, write) = socket.into_split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

    let writer_future = tokio::spawn(async move {
        let mut write = write;

        while let Some(buffer) = rx.recv().await {
            let packet = match encode_packet(&buffer) {
                Ok(p) => p,

                Err(e) => {
                    crate::log::log(
                        LogLevel::Error,
                        format!("Failed to encode packet: {}", e).as_str(),
                    );
                    break;
                }
            };

            if let Err(_) = write.write_all(&packet).await {
                break;
            }

            while let Ok(buffer) = rx.try_recv() {
                let packet = match encode_packet(&buffer) {
                    Ok(p) => p,

                    Err(e) => {
                        crate::log::log(
                            LogLevel::Error,
                            format!("Failed to encode packet: {}", e).as_str(),
                        );
                        break;
                    }
                };

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
                        format!("Sent keep alive to {}", username).as_str(),
                    );
                }
            }
        })
    };

    if player.x == 0.0 && player.y == 0.0 && player.z == 0.0 {
        let region_arc = get_region(player.x as i32 >> 9, player.z as i32 >> 9).await;
        let cx = player.x as i32 >> 4;
        let cz = player.z as i32 >> 4;
        let chunk = get_chunk(&region_arc, cx, cz).await;
        let surface_y =
            chunk.get_surface_y((player.x as i32 & 15) as u8, (player.z as i32 & 15) as u8);

        player.y = surface_y as f64 + 1.0;
    }

    let mut buffer = Vec::new();
    send_commands(&mut buffer, player.is_op).await?;
    let _ = tx.send(buffer);

    let mut buffer = Vec::new();
    send_set_ticking_state(&mut buffer).await?;
    let _ = tx.send(buffer);

    broadcast_system_message(format!(
        "{}{} has joined the server",
        colors::YELLOW,
        player.username
    ))
    .await?;

    let player = Arc::new(Mutex::new(player));

    {
        let player_lock = player.lock().await;

        PLAYER_SOCKET_MAP
            .write()
            .await
            .insert(player_lock.uuid.clone(), tx.clone());

        PLAYERS
            .write()
            .await
            .insert(player_lock.uuid.clone(), Arc::clone(&player));

        NAME_TO_UUID.write().await.insert(
            player_lock.username.clone().to_lowercase(),
            player_lock.uuid.clone(),
        );
    }

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
                vec![
                    PlayerAction::AddPlayer,
                    PlayerAction::UpdateGameMode,
                    PlayerAction::UpdateListed(true),
                ],
            )
            .await?;

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
            vec![PlayerAction::AddPlayer, PlayerAction::UpdateListed(true)],
        )
        .await?;

        let _ = player_tx.send(buffer);
    }

    crate::log::log(
        LogLevel::Debug,
        format!(
            "Sent player info updates for {}",
            player.lock().await.username
        )
        .as_str(),
    );

    let chunk_sender_task = {
        let tx = tx.clone();
        let player_arc = Arc::clone(&player);

        tokio::spawn(async move {
            let _ = send_chunks_to_player(tx, player_arc).await;
        })
    };

    {
        let mut player_guard = player.lock().await;
        player_guard.sync_player_position().await?;
        player_guard.sync_player_inventory().await?;
        player_guard.sync_player_health().await?;

        crate::log::log(
            LogLevel::Debug,
            format!("Synchronized intial data for {}", player_guard.username).as_str(),
        );

        player_guard
            .send_system_message(format!(
                "This server is running Mist {}",
                env!("CARGO_PKG_VERSION")
            ))
            .await?;

        player_guard
            .send_system_message(
                "Please report any bugs at https://github.com/cyteon/mist/issues/new".to_string(),
            )
            .await?;
    }

    loop {
        match timeout(
            Duration::from_secs(30),
            read_packet(&mut read, &ProtocolState::Play, true),
        )
        .await
        {
            Ok(Ok(Some(packet))) => match packet {
                ClientPacket::ConfirmTeleprortion(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_confirm_teleportation(&mut cursor, &mut player).await?;
                }

                ClientPacket::PlayerAction(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_player_action(&mut cursor, &mut player).await?
                }

                ClientPacket::UseItemOn(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_use_item_on(&mut cursor, &mut player).await?;
                }

                ClientPacket::UseItem(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_use_item(&mut cursor, &mut player).await?;
                }

                ClientPacket::ChatCommand(mut cursor) => {
                    let command = read_chat_command(&mut cursor).await?;

                    crate::log::log(
                        LogLevel::Info,
                        format!("{} issued command /{}", username, command).as_str(),
                    );

                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    handle_command(
                        command,
                        &mut CommandInvoker::Player {
                            player: &mut *player,
                        },
                    )
                    .await?;
                }

                ClientPacket::ChatMessage(mut cursor) => {
                    let message = read_chat_message(&mut cursor).await?;

                    let arc = Arc::clone(&player);
                    let player_clone = arc.lock().await.clone();

                    crate::log::log(
                        LogLevel::Info,
                        format!("<{}> {}", player_clone.username, message.content).as_str(),
                    );

                    for player in PLAYERS.read().await.values() {
                        let mut target_player_guard = player.lock().await;

                        if target_player_guard.uuid == uuid {
                            let mut buffer = Vec::new();

                            send_player_chat_message(
                                &mut buffer,
                                &player_clone,
                                &mut *target_player_guard,
                                &message,
                            )
                            .await?;

                            let _ = tx.send(buffer);
                        } else {
                            let player_sockets = PLAYER_SOCKET_MAP.read().await;
                            let target_player_tx =
                                player_sockets.get(&target_player_guard.uuid).unwrap();

                            let mut buffer = Vec::new();

                            send_player_chat_message(
                                &mut buffer,
                                &player_clone,
                                &mut *target_player_guard,
                                &message,
                            )
                            .await?;

                            let _ = target_player_tx.send(buffer);
                        }
                    }
                }

                ClientPacket::SetPlayerPosition(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_set_player_position(&mut cursor, &mut player).await?;
                }

                ClientPacket::SetPlayerPositionAndRotation(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_set_player_position_and_rotation(&mut cursor, &mut player).await?;
                }

                ClientPacket::PlayerInput(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_player_input(&mut cursor, &mut player).await?;
                }

                ClientPacket::SetPlayerRotation(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_set_player_rotation(&mut cursor, &mut player).await?;
                }

                ClientPacket::SetCarriedItem(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_set_carried_item(&mut cursor, &mut player).await?;
                }

                ClientPacket::SetCreativeModeSlot(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_set_creative_mode_slot(&mut cursor, &mut player).await?;
                }

                ClientPacket::PickItemFromBlock(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_pick_item_from_block(&mut cursor, &mut player).await?;
                }

                ClientPacket::PlayerAbilities(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_player_abilities(&mut cursor, &mut player).await?;
                }

                ClientPacket::ClientStatus(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    let action = read_client_status(&mut cursor, &mut player).await?;
                    let player_id = player.id;
                    drop(player);

                    if action == 0 {
                        let players = PLAYERS.read().await;
                        for (other_uuid, p) in players.iter() {
                            if other_uuid == &uuid {
                                continue;
                            }

                            p.lock().await.loaded_entities.retain(|&id| id != player_id);
                        }

                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            send_chunks_to_player(tx_clone, arc.clone()).await.unwrap();
                        });
                    }
                }

                ClientPacket::SwingArm(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_swing_arm(&mut cursor, &mut player).await?;
                }

                ClientPacket::ContainerClick(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_container_click(&mut cursor, &mut player).await?;
                }

                ClientPacket::ContainerClose(mut cursor) => {
                    let arc = Arc::clone(&player);
                    let mut player = arc.lock().await;

                    read_container_close(&mut cursor, &mut player).await?;
                }

                _ => {}
            },

            Ok(Ok(None)) => {}

            Err(e) => {
                crate::log::log(
                    LogLevel::Error,
                    format!("{} has timed out during play state: {}", username, e).as_str(),
                );

                break;
            }

            Ok(Err(e)) => {
                crate::log::log(
                    LogLevel::Error,
                    format!(
                        "Error while reading packet from {} during play state: {}",
                        username, e
                    )
                    .as_str(),
                );

                break;
            }
        }
    }

    crate::log::log(
        LogLevel::Info,
        format!("{} ({}) left the server", username, uuid).as_str(),
    );

    PLAYER_SOCKET_MAP.write().await.remove(&uuid);

    drop(tx);

    writer_future.abort();
    keep_alive_future.abort();
    chunk_sender_task.abort();

    if PLAYERS.read().await.len() == 1 {
        crate::log::log(
            LogLevel::Debug,
            "No players online, saving and clearing regions from memory",
        );

        save::save().await?;
        REGIONS.lock().await.clear(); // unnecesary having all regions loaded in while nobody is playing
    } else {
        let players_locked = PLAYERS.read().await;
        let player_lock = players_locked.get(&uuid).unwrap().clone();
        save::save_player(&player_lock.lock().await.clone()).await?;

        for other_tx in PLAYER_SOCKET_MAP.read().await.values().into_iter() {
            let mut buffer = Vec::new();
            send_player_info_remove(&mut buffer, vec![&uuid]).await?;

            let _ = other_tx.send(buffer);
        }
    }

    {
        let mut players_guard = PLAYERS.write().await;
        let player_guard = players_guard.remove(&uuid);

        if let Some(player) = player_guard {
            let player_lock = player.lock().await;

            let mut entities_write = ENTITIES.write().await;
            entities_write.remove(&player_lock.id);
            drop(entities_write);

            if let Some(window) = player_lock.current_window {
                let uuid = player_lock.uuid.clone();
                drop(player_lock);

                match window {
                    WindowType::Chest { cords, .. } => {
                        let chunk_pos = (cords.0.div_euclid(16), cords.2.div_euclid(16));
                        let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

                        let region = get_region(region_pos.0, region_pos.1).await;
                        let mut region_lock = region.lock().await;

                        match region_lock.get_chunk(chunk_pos.0, chunk_pos.1) {
                            Some(chunk) => {
                                if let Some(be) = chunk.block_entities.get_mut(&(
                                    cords.0 & 15,
                                    cords.1,
                                    cords.2 & 15,
                                )) {
                                    match be {
                                        BlockEntityData::Chest { viewers, .. } => {
                                            viewers.retain(|viewer| viewer != &uuid);

                                            let mut buffer = Vec::new();
                                            send_block_action(
                                                &mut buffer,
                                                cords,
                                                1,
                                                viewers.len() as u8,
                                            )
                                            .await?;
                                            broadcast_packet(
                                                buffer,
                                                (cords.0 as f64, cords.1 as f64, cords.2 as f64),
                                                Some(uuid.clone()),
                                            )
                                            .await?;
                                        }

                                        _ => {}
                                    }
                                }
                            }

                            None => {}
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    PLAYERS.write().await.remove(&uuid);
    NAME_TO_UUID.write().await.remove(&username.to_lowercase());
    PLAYER_POSITIONS.write().await.remove(&uuid);

    broadcast_system_message(format!(
        "{}{} has left the server",
        colors::YELLOW,
        username
    ))
    .await?;

    Ok(())
}
