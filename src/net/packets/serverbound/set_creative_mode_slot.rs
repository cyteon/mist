use tokio::io::AsyncReadExt;

use crate::net::codec::read_slot;
use crate::types::player::Player;

pub async fn read_set_creative_mode_slot<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let slot = stream.read_i16().await?;
    let item = read_slot(stream).await?;

    player.inventory[slot as usize] = item;

    if let Some(item) = item {
        println!(
            "Player {} set creative mode slot {} to item {} (count {})",
            player.username, slot, item.item_id, item.count
        );
    }

    Ok(())
}
