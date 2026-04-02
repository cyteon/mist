use tokio::io::AsyncReadExt;
use crate::types::player::Player;

pub async fn read_set_player_position<R: AsyncReadExt + Unpin>(
    stream: &mut R, player: &mut Player
) -> anyhow::Result<()> {
    let x = stream.read_f64().await?;
    let y = stream.read_f64().await?;
    let z = stream.read_f64().await?;

    player.x = x;
    player.y = y;
    player.z = z;

    let flags = stream.read_u8().await?;
    player.on_ground = flags & 0x01 != 0;

    Ok(())
}