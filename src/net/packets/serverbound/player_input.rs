use tokio::io::AsyncReadExt;
use crate::types::player::Player;

pub async fn read_player_input
<R: AsyncReadExt + Unpin>(
    stream: &mut R, player: &mut Player
) -> anyhow::Result<()> {
    let flags = stream.read_u8().await?;

    player.movement.forward = flags & 0x01 != 0;
    player.movement.backward = flags & 0x02 != 0;
    player.movement.left = flags & 0x04 != 0;
    player.movement.right = flags & 0x08 != 0;
    player.movement.jumping = flags & 0x10 != 0;
    player.movement.sneaking = flags & 0x20 != 0;
    player.movement.sprinting = flags & 0x40 != 0;

    Ok(())
}