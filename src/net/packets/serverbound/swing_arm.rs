use crate::net::codec::read_var;
use crate::types::player::Player;
use tokio::io::AsyncReadExt;

pub async fn read_swing_arm<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let hand = read_var(stream).await?;
    player.send_hand_swing(hand == 0).await?;

    Ok(())
}
