use tokio::io::AsyncWriteExt;
use crate::net::codec::write_var;

pub async fn send_keep_alive<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::KEEP_ALIVE as u8];

    let ms = chrono::Utc::now().timestamp_millis();
    packet_data.write_i64(ms).await?;

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}