use tokio::io::AsyncReadExt;

use crate::{net::codec::{read_position, read_var}, world::get_region};

pub enum ActionStatus {
    StartedDigging = 0,
    //CancelledDigging = 1,
    FinishedDigging = 2,
    //DropItemStack = 3,
    //DropItem = 4,
    //ShootArrowFinishEating = 5,
    //SwapItemInHand = 6,
}

pub async fn read_player_action<R: AsyncReadExt + Unpin>(stream: &mut R, player: &mut crate::types::player::Player) -> anyhow::Result<Option<crate::types::entity::Entity>> {
    let status = read_var(stream).await?;

    let (x, y, z) = read_position(stream).await?;

    let _face = stream.read_u8().await?;
    let _sequence = read_var(stream).await?;

    let instant_dig = status == ActionStatus::StartedDigging as u32 && player.gamemode as u8 == 1;

    if instant_dig || status == ActionStatus::FinishedDigging as u32 {
        let chunk_pos = (x.div_euclid(16), z.div_euclid(16));
        let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

        let _section_y = y.div_euclid(16) + 4; // cause section 0 is -64

        let regions_lock = get_region(region_pos.0, region_pos.1).await;
        let mut region = regions_lock.lock().await;

        if let Some(chunk) = region.chunks.iter_mut().find(|chunk| chunk.x == chunk_pos.0 && chunk.z == chunk_pos.1) {
            let spawned_entity = if player.gamemode as u8 != 1 {
                Some(crate::types::entity::spawn_item_drop(
                    crate::types::items::ItemStack {
                        item_id: crate::types::items::block_to_item_id(chunk.get_block((x & 15) as u8, y as i32, (z & 15) as u8) as i32).unwrap_or(0) as i32,
                        count: 1,
                    },
                    x as f64 + 0.5,
                    y as f64 + 0.5,
                    z as f64 + 0.5,
                ))
            } else {
                None
            };

            chunk.set_block((x & 15) as u8, y as i32, (z & 15) as u8, 0);

            return Ok(spawned_entity);
        }
    }

    Ok(None)
}