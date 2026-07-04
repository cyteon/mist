pub async fn send_update_time<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    timestamp: i64,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::SET_TIME as u8];

    packet_data.extend_from_slice(&timestamp.to_be_bytes());
    packet_data.extend_from_slice(&(timestamp % 24000).to_be_bytes());
    packet_data.push(true as u8); // time of day increasing

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
