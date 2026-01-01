use tokio::io::AsyncReadExt;
use crate::types::player::Player;

pub async fn read_player_input
<R: AsyncReadExt + Unpin>(
    stream: &mut R, player: &mut Player
) -> anyhow::Result<()> {
    let flags = stream.read_u8().await?;

    if flags & 0x01 != 0 {
        player.movement.foward = !player.movement.foward;
    }

    if flags & 0x02 != 0 {
        player.movement.backward = !player.movement.backward;
    }

    if flags & 0x04 != 0 {
        player.movement.left = !player.movement.left;
    }

    if flags & 0x08 != 0 {
        player.movement.right = !player.movement.right;
    }

    if flags & 0x10 != 0 {
        player.movement.jumping = !player.movement.jumping;
    }

    if flags & 0x20 != 0 {
        player.movement.sneaking = !player.movement.sneaking;
    }

    if flags & 0x40 != 0 {
        player.movement.sprinting = !player.movement.sprinting;
    }

    Ok(())
}