use crate::net::codec::write_var;
use tokio::io::AsyncWriteExt;

pub async fn send_game_event<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, event: u8, value: f32) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::GAME_EVENT as u8];

    packet_data.push(event);
    packet_data.write_f32(value).await?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}