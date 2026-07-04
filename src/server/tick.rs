use fancy_log::LogLevel;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use tokio::time;
use tokio::time::Duration;

use crate::server::save::save;

pub static TPS_5S: AtomicU32 = AtomicU32::new(20);
pub static TPS_1M: AtomicU32 = AtomicU32::new(20);
pub static TPS_5M: AtomicU32 = AtomicU32::new(20);

pub static TIMESTAMP: AtomicI64 = AtomicI64::new(0);

pub async fn start_tick_loop() -> anyhow::Result<()> {
    crate::log::log(LogLevel::Info, "Tick loop started");

    let world_save = crate::server::save::load_world_data();
    TIMESTAMP.store(world_save.timestamp, Ordering::Relaxed);

    let mut interval = time::interval(Duration::from_millis(50)); // 20 tps
    let mut ticks_until_autosave = 100; // so it autosaves 5 seconds after start

    let mut last_tps_5s_check = std::time::Instant::now();
    let mut ticks_5s = 0;

    let mut last_tps_1m_check = std::time::Instant::now();
    let mut ticks_1m = 0;

    let mut last_tps_5m_check = std::time::Instant::now();
    let mut ticks_5m = 0;

    loop {
        TIMESTAMP.fetch_add(1, Ordering::Relaxed);

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
            if let crate::types::entity::EntityType::Item(item_entity) = &entity.entity_type {
                if item_entity.dropped_at.elapsed().as_millis() < 500 {
                    continue;
                }

                for player in players.values() {
                    let player = player.lock().await;

                    if let Some(dropped_by) = &item_entity.dropped_by {
                        if *dropped_by == player.uuid
                            && item_entity.dropped_at.elapsed().as_secs() < 2
                        {
                            continue;
                        }
                    }

                    if player.dead || !player.initial_sync_done {
                        continue;
                    }

                    let distance_squared = (player.x - entity.x).powi(2)
                        + (player.y - entity.y).powi(2)
                        + (player.z - entity.z).powi(2);

                    if distance_squared < 1.5 * 1.5 {
                        to_pickup.push((
                            entity.id,
                            player.uuid.clone(),
                            item_entity.item_stack.count,
                        ));
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
                if let crate::types::entity::EntityType::Item(item_entity) = entity.entity_type {
                    let players = crate::server::state::play::PLAYERS.read().await;

                    if let Some(player) = players.get(&player_uuid) {
                        let mut player_lock = player.lock().await;
                        player_lock
                            .give_item(
                                item_entity.item_stack.item_id,
                                item_entity.item_stack.count as i32,
                            )
                            .await?;

                        let mut buffer = Vec::new();
                        crate::net::packets::clientbound::pickup_item::send_pickup_item(
                            &mut buffer,
                            entity_id,
                            player_lock.id,
                            count as i32,
                        )
                        .await
                        .unwrap();

                        player_lock.send_packet(buffer).await?;
                    }

                    drop(players);
                }
            }
        }

        interval.tick().await;
    }
}
