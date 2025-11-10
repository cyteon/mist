use tokio::io::AsyncWriteExt;
use crate::net::codec::write_var;

pub async fn send_keep_alive<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![0x26];

    let ms = chrono::Utc::now().timestamp_millis();
    packet_data.write_i64(ms).await?;

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}