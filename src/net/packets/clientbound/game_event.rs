use crate::net::codec::write_var;
use tokio::io::AsyncWriteExt;

pub async fn send_game_event<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, event: u8, value: f32) -> anyhow::Result<()> {
    let mut packet_data = vec![0x26];

    packet_data.push(event);
    packet_data.write_f32(value).await?;

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}