use crate::net::codec::read_var;
use crate::net::packets::clientbound::block_action::send_block_action;
use crate::types::block_entities::BlockEntityData;
use crate::types::player::{Player, WindowType};
use crate::world::get_region;
use tokio::io::AsyncReadExt;

pub async fn read_container_close<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let _window_id = read_var(stream).await?;

    match player.current_window {
        Some(window) => match window {
            WindowType::CraftingTable(grid) => {
                for i in 1..=9 {
                    if let Some(item_stack) = grid[i] {
                        player
                            .give_item(item_stack.item_id, item_stack.count as i32)
                            .await?;
                    }
                }
            }

            WindowType::Chest { cords, .. } => {
                let chunk_pos = (cords.0.div_euclid(16), cords.2.div_euclid(16));
                let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

                let region = get_region(region_pos.0, region_pos.1).await;
                let mut region_lock = region.lock().await;

                if let Some(chunk) = region_lock.get_chunk(chunk_pos.0, chunk_pos.1) {
                    if let Some(be) =
                        chunk
                            .block_entities
                            .get_mut(&(cords.0 & 15, cords.1, cords.2 & 15))
                    {
                        if let BlockEntityData::Chest { viewers, .. } = be {
                            viewers.retain(|viewer| viewer != &player.uuid);

                            dbg!("Chest at {:?} now has {} viewers", cords, viewers.len());

                            let mut buffer = Vec::new();
                            send_block_action(&mut buffer, cords, 1, viewers.len() as u8).await?;
                            player.send_packet(buffer).await?;
                        }
                    }
                }
            }
        },

        None => {}
    }

    player.current_window = None;

    Ok(())
}
