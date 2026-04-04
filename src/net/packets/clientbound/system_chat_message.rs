pub async fn send_system_chat_message<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    message: String,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::SYSTEM_CHAT as u8];

    craftflow_nbt::to_writer(&mut packet_data, &craftflow_nbt::DynNBT::String(message))?;

    packet_data.push(0x00); // overlay

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
