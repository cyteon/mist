pub async fn send_entity_event<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    entity_id: i32,
    status: u8,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::ENTITY_EVENT as u8];

    packet_data.extend_from_slice(&entity_id.to_be_bytes());
    packet_data.push(status);

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
