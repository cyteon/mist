use crate::types::player::Player;
use tokio::io::AsyncReadExt;

pub async fn read_set_player_rotation<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let yaw = stream.read_f32().await?;
    let pitch = stream.read_f32().await?;

    player.yaw = yaw;
    player.pitch = pitch;

    Ok(())
}
