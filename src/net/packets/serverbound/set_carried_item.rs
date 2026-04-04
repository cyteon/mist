use crate::types::player::Player;
use tokio::io::AsyncReadExt;

pub async fn read_set_carried_item<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let slot = stream.read_i16().await?;
    player.current_slot = slot;

    Ok(())
}
