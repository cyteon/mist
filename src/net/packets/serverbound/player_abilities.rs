use crate::types::player::Player;
use tokio::io::AsyncReadExt;

pub async fn read_player_abilities<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let flags = stream.read_u8().await?;
    player.flying = flags & 0x02 != 0;

    Ok(())
}
