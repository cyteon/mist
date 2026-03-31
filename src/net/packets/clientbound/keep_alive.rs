use tokio::io::AsyncWriteExt;

pub async fn send_keep_alive<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::KEEP_ALIVE as u8];

    let ms = chrono::Utc::now().timestamp_millis();
    packet_data.write_i64(ms).await?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}