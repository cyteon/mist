use crate::net::codec::read_var;
use crate::types::player::Player;
use tokio::io::AsyncReadExt;

pub async fn read_container_close<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let _window_id = read_var(stream).await?;
    player.current_window = None;

    Ok(())
}
