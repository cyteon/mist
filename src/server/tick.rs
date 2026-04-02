use tokio::time;
use fancy_log::LogLevel;
use tokio::time::Duration;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::server::save::save;

pub static TPS_5S: AtomicU32 = AtomicU32::new(20);
pub static TPS_1M: AtomicU32 = AtomicU32::new(20);
pub static TPS_5M: AtomicU32 = AtomicU32::new(20);

pub async fn start_tick_loop() -> anyhow::Result<()> {
    crate::log::log(LogLevel::Info, "Tick loop started");

    let mut interval = time::interval(Duration::from_millis(50)); // 20 tps
    let mut ticks_until_autosave = 100; // so it autosaves 5 seconds after start

    let mut last_tps_5s_check = std::time::Instant::now();
    let mut ticks_5s = 0;

    let mut last_tps_1m_check = std::time::Instant::now();
    let mut ticks_1m = 0;

    let mut last_tps_5m_check = std::time::Instant::now();
    let mut ticks_5m = 0;

    loop {
        if ticks_until_autosave == 0 {
            ticks_until_autosave = 6000; // 5 mins
            save().await;
        } else {
            ticks_until_autosave -= 1;
        }

        ticks_5s += 1;
        if last_tps_5s_check.elapsed().as_secs() >= 5 {
            let elapsed = last_tps_5s_check.elapsed().as_secs_f64();
            let tps = ticks_5s as f64 / elapsed;
            TPS_5S.store(tps.round() as u32, Ordering::Relaxed);

            last_tps_5s_check = std::time::Instant::now();
            ticks_5s = 0;
        }

        ticks_1m += 1;
        if last_tps_1m_check.elapsed().as_secs() >= 60 {
            let elapsed = last_tps_1m_check.elapsed().as_secs_f64();
            let tps = ticks_1m as f64 / elapsed;
            TPS_1M.store(tps.round() as u32, Ordering::Relaxed);

            last_tps_1m_check = std::time::Instant::now();
            ticks_1m = 0;
        }

        ticks_5m += 1;
        if last_tps_5m_check.elapsed().as_secs() >= 300 {
            let elapsed = last_tps_5m_check.elapsed().as_secs_f64();
            let tps = ticks_5m as f64 / elapsed;
            TPS_5M.store(tps.round() as u32, Ordering::Relaxed);

            last_tps_5m_check = std::time::Instant::now();
            ticks_5m = 0;
        }

        let players = crate::server::state::play::PLAYERS.read().await;

        for player in players.values() {
            let mut player_lock = player.lock().await;
            player_lock.tick().await?;
        }

        let mut entities = crate::types::entity::ENTITIES.write().await;
        for entity in entities.values_mut() {
            entity.tick().await?;
        }

        let mut to_pickup = Vec::new();

        for entity in entities.values() {
            if let crate::types::entity::EntityType::Item(item_stack) = &entity.entity_type {
                for player in players.values() {
                    let player = player.lock().await;

                    let distance_squared = (player.x - entity.x).powi(2) + (player.y - entity.y).powi(2) + (player.z - entity.z).powi(2);

                    if distance_squared < 1.5 * 1.5 {
                        to_pickup.push((entity.id, player.uuid.clone(), item_stack.count));
                        break;
                    }
                }
            }
        }

        drop(players);
        drop(entities);

        for (entity_id, player_uuid, count) in to_pickup {
            let mut entities = crate::types::entity::ENTITIES.write().await;

            if let Some(entity) = entities.remove(&entity_id) {
                if let crate::types::entity::EntityType::Item(item_stack) = entity.entity_type {
                    let players = crate::server::state::play::PLAYERS.read().await;

                    if let Some(player) = players.get(&player_uuid) {
                        let mut player_lock = player.lock().await;
                        player_lock.give_item(item_stack.item_id, count as i32).await?;

                        let tx = crate::server::conn::PLAYER_SOCKET_MAP.read().await.get(&player_lock.uuid).cloned().unwrap();

                        let mut buffer = Vec::new();
                        crate::net::packets::clientbound::pickup_item::send_pickup_item(&mut buffer, entity_id, player_lock.id, count as i32).await.unwrap();
                        let _ = tx.send(buffer);
                    }

                    drop(players);

                    entity.broadcast_despawn().await;
                }
            }
        }

        interval.tick().await;
    }
}