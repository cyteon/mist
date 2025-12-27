use tokio::io::AsyncReadExt;

use crate::{net::codec::{read_position, read_var}, types::player::Player};

pub async fn read_use_item_on<R: AsyncReadExt + Unpin>(stream: &mut R, player: &mut Player) -> anyhow::Result<()> {
    let hand = read_var(stream).await?;
    let (x, y, z) = read_position(stream).await?;

    let face = stream.read_u8().await?;

    let cursor_x = stream.read_f32().await?;
    let cursor_y = stream.read_f32().await?;
    let cursor_z = stream.read_f32().await?;
    
    let inside_block = stream.read_u8().await?;
    let world_border_hit = stream.read_u8().await?;
    let sequence = read_var(stream).await?;

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

    // todo: replace stone placeholder
    let block_id = crate::types::blocks::get("minecraft:stone").unwrap().id;

    let chunk_pos = (bx.div_euclid(16), bz.div_euclid(16));
    let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

    let section_y = by.div_euclid(16) + 4; // cause section 0 is -64

    let mut regions_lock = crate::world::worldgen::REGIONS.lock().await;
    if let Some(region) = regions_lock.get_mut(&region_pos) {
        if let Some(chunk) = region.chunks.iter_mut().find(|chunk| chunk.x == chunk_pos.0 && chunk.z == chunk_pos.1) {
            if let Some(section) = chunk.sections.iter_mut().find(|section| section.y == section_y) {
                section.set_block((bx & 15) as u8, (by & 15) as u8, (bz & 15) as u8, block_id);
            }
        }
    }

    Ok(())
}