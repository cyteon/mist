use tokio::io::AsyncReadExt;

use crate::{net::codec::{read_position, read_var}, types::player::Player, world::get_region};

pub async fn read_use_item_on<R: AsyncReadExt + Unpin>(stream: &mut R, player: &mut Player) -> anyhow::Result<()> {
    let _hand = read_var(stream).await?;
    let (x, y, z) = read_position(stream).await?;

    let face = stream.read_u8().await?;

    let _cursor_x = stream.read_f32().await?;
    let _cursor_y = stream.read_f32().await?;
    let _cursor_z = stream.read_f32().await?;
    
    let _inside_block = stream.read_u8().await?;
    let _world_border_hit = stream.read_u8().await?;
    let _sequence = read_var(stream).await?;

    let (mut bx, mut by, mut bz) = (x, y, z);

    match face {
        0 => by -= 1,
        1 => by += 1,
        2 => bz -= 1,
        3 => bz += 1,
        4 => bx -= 1,
        5 => bx += 1,
        _ => {}
    }

    let block_id = if let Some(Some(item_stack)) = player.inventory.get(player.current_slot as usize + 36) {
        if let Some(block_id) = crate::types::items::item_to_block(item_stack.item_id) {
            block_id
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    let chunk_pos = (bx.div_euclid(16), bz.div_euclid(16));
    let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

    let section_y = by.div_euclid(16) + 4; // cause section 0 is -64

    let region = get_region(region_pos.0, region_pos.1).await;
    let mut region_lock = region.lock().await;

    if let Some(chunk) = region_lock.chunks.iter_mut().find(|chunk| chunk.x == chunk_pos.0 && chunk.z == chunk_pos.1) {
        if let Some(section) = chunk.sections.iter_mut().find(|section| section.y == section_y) {
            section.set_block((bx & 15) as u8, (by & 15) as u8, (bz & 15) as u8, block_id as u16);

            if player.gamemode as u8 != 1 {
                player.inventory[player.current_slot as usize + 36].as_mut().unwrap().count -= 1;

                if player.inventory[player.current_slot as usize + 36].as_ref().unwrap().count == 0 {
                    player.inventory[player.current_slot as usize + 36] = None;
                }
            }
        }
    }

    Ok(())
}