use crate::types::player::Player;
use tokio::io::AsyncReadExt;

pub async fn read_set_player_position_and_rotation<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let x = stream.read_f64().await?;
    let y = stream.read_f64().await?;
    let z = stream.read_f64().await?;

    player.x = x;
    player.y = y;
    player.z = z;

    let yaw = stream.read_f32().await?;
    let pitch = stream.read_f32().await?;

    player.yaw = yaw;
    player.pitch = pitch;

    let flags = stream.read_u8().await?;
    player.on_ground = flags & 0x01 != 0;

    Ok(())
}
