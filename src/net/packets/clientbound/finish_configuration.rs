use crate::net::codec::write_var;

pub async fn send_finish_configuration<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let packet_data = vec![crate::net::packet::configuration::clientbound::FINISH_CONFIGURATION as u8];

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}