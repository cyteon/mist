use crate::net::codec::write_var;

pub async fn send_finish_configuration<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let packet_data = vec![crate::net::packet::configuration::clientbound::FINISH_CONFIGURATION as u8];

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}