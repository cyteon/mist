use tokio::io::AsyncReadExt;

use crate::{
    net::codec::read_position,
    types::{
        items::{ItemStack, block_to_item},
        player::{Gamemode, Player},
    },
    world::get_region,
};

pub async fn read_pick_item_from_block<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let (x, y, z) = read_position(stream).await?;
    let _include_data = stream.read_u8().await? != 0;

    let chunk_pos = (x.div_euclid(16), z.div_euclid(16));
    let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

    let region = get_region(region_pos.0, region_pos.1).await;
    let mut region_lock = region.lock().await;

    if let Some(chunk) = region_lock.get_chunk(chunk_pos.0, chunk_pos.1) {
        let block = chunk.get_block((x & 15) as u8, y as i32, (z & 15) as u8);

        if let Some(id) = block_to_item(block) {
            if player.gamemode == Gamemode::Creative {
                player.inventory[player.current_slot as usize + 36] = Some(ItemStack {
                    item_id: id,
                    count: 1,
                });

                player.set_inventory_slot(player.current_slot + 36).await?;
            }
        }
    }

    Ok(())
}
