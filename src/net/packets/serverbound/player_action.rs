use tokio::io::AsyncReadExt;

use crate::{
    net::codec::{read_position, read_var},
    world::get_region,
};

pub enum ActionStatus {
    StartedDigging = 0,
    //CancelledDigging = 1,
    FinishedDigging = 2,
    DropItemStack = 3,
    DropItem = 4,
    //ShootArrowFinishEating = 5,
    //SwapItemInHand = 6,
}

pub async fn read_player_action<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut crate::types::player::Player,
) -> anyhow::Result<()> {
    let status = read_var(stream).await?;

    let (x, y, z) = read_position(stream).await?;

    let _face = stream.read_u8().await?;
    let sequence = read_var(stream).await?;

    let instant_dig = status == ActionStatus::StartedDigging as u32 && player.gamemode as u8 == 1;

    if instant_dig || status == ActionStatus::FinishedDigging as u32 {
        let chunk_pos = (x.div_euclid(16), z.div_euclid(16));
        let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

        let regions_lock = get_region(region_pos.0, region_pos.1).await;
        let mut region = regions_lock.lock().await;

        if let Some(chunk) = region
            .chunks
            .iter_mut()
            .find(|chunk| chunk.x == chunk_pos.0 && chunk.z == chunk_pos.1)
        {
            if player.gamemode as u8 != 1 {
                crate::types::entity::spawn_item_drop(
                    crate::types::items::ItemStack {
                        item_id: crate::types::items::block_to_item_id(chunk.get_block(
                            (x & 15) as u8,
                            y,
                            (z & 15) as u8,
                        )
                            as i32)
                        .unwrap_or(0) as i32,
                        count: 1,
                    },
                    None,
                    x as f64 + 0.5,
                    y as f64 + 0.5,
                    z as f64 + 0.5,
                );
            }

            chunk.set_block((x & 15) as u8, y, (z & 15) as u8, 0);

            let tx = crate::server::conn::PLAYER_SOCKET_MAP
                .read()
                .await
                .get(&player.uuid)
                .cloned()
                .unwrap();

            let mut buffer = Vec::new();
            crate::net::packets::clientbound::block_changed_ack::send_block_changed_ack(
                &mut buffer,
                sequence as i32,
            )
            .await?;
            let _ = tx.send(buffer);

            crate::net::packets::clientbound::block_update::broadcast_block_update(x, y, z, 0)
                .await?;
        }
    } else if status == ActionStatus::DropItemStack as u32
        || status == ActionStatus::DropItem as u32
    {
        let drop_all = status == ActionStatus::DropItemStack as u32;
        let held_slot = player.current_slot as usize + 36;

        if let Some(item) = &mut player.inventory[held_slot] {
            let count = if drop_all { item.count } else { 1 };
            let item_id = item.item_id;

            item.count -= count;
            if item.count == 0 {
                player.inventory[held_slot] = None;
            }

            let (dx, dz) = {
                let yaw = player.yaw.to_radians();
                (-yaw.sin() as f64, yaw.cos() as f64)
            };

            crate::types::entity::spawn_item_drop(
                crate::types::items::ItemStack { item_id, count },
                Some(player.uuid.clone()),
                player.x + dx,
                player.y + 1.0,
                player.z + dz,
            );
        }
    }

    Ok(())
}
