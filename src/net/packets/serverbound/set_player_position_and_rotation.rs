use tokio::io::AsyncReadExt;
use crate::types::player::Player;

pub async fn read_set_player_position_and_rotation<R: AsyncReadExt + Unpin>(
    stream: &mut R, player: &mut Player
) -> anyhow::Result<()> {
    let x = stream.read_f64().await?;
    let y = stream.read_f64().await?;
    let z = stream.read_f64().await?;

    player.x = x;
    player.y = y;
    player.z = z;

    // todo: yaw, pitch, flags (on ground, pushing against wall)

    Ok(())
}