use crate::{net::codec::write_var, types::player::Player};

pub async fn send_player_info_update<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, players: Vec<&Player>) -> anyhow::Result<()> {
    let mut packet_data = vec![0x44];

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}