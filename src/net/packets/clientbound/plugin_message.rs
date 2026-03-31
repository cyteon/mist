use crate::net::codec::write_string;

pub async fn send_plugin_message<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::configuration::clientbound::CUSTOM_PAYLOAD as u8];

    write_string(&mut packet_data, "minecraft:brand")?;
    write_string(&mut packet_data, "mist")?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}