use crate::net::codec::write_var;

pub async fn send_pong<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![0x01];
    packet_data.extend_from_slice([0u8; 8].as_ref());

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}