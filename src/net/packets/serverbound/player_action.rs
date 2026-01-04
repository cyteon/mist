use tokio::io::AsyncReadExt;

use crate::{net::codec::{read_position, read_var}, world::worldgen::get_region};

pub async fn read_player_action<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<()> {
    let status = read_var(stream).await?;

    let (x, y, z) = read_position(stream).await?;

    let _face = stream.read_u8().await?;
    let _sequence = read_var(stream).await?;

    if status == 0 || status == 2 {
        let chunk_pos = (x.div_euclid(16), z.div_euclid(16));
        let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

        let section_y = y.div_euclid(16) + 4; // cause section 0 is -64

        let regions_lock = get_region(region_pos.0, region_pos.1).await;
        let mut region = regions_lock.lock().await;

        if let Some(chunk) = region.chunks.iter_mut().find(|chunk| chunk.x == chunk_pos.0 && chunk.z == chunk_pos.1) {
            if let Some(section) = chunk.sections.iter_mut().find(|section| section.y == section_y) {
                section.set_block((x & 15) as u8, (y & 15) as u8, (z & 15) as u8, 0);
            }
        }
    }

    Ok(())
}