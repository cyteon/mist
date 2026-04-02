use tokio::io::AsyncReadExt;
use crate::types::player::Player;
use crate::net::codec::read_var;

pub async fn read_client_status<R: AsyncReadExt + Unpin>(
    stream: &mut R, player: &mut Player
) -> anyhow::Result<u32> {
    let action_id = read_var(stream).await?;

    match action_id {
        0 => {
            player.respawn().await?;
        },

        _ => {}
    }

    Ok(action_id)
}