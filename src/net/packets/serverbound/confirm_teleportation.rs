use tokio::io::AsyncReadExt;

use crate::{net::codec::read_var, types::player::Player};

pub async fn read_confirm_teleportation<R: AsyncReadExt + Unpin>(stream: &mut R, player: &mut Player) -> anyhow::Result<()> {
    let teleportation_id = read_var(stream).await?;

    if teleportation_id == 1 {
        player.initial_sync_done = true;
    }

    Ok(())
}