use crate::net::codec::write_var;

pub async fn send_pong<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::status::clientbound::PONG_RESPONSE as u8];

    packet_data.extend_from_slice([0u8; 8].as_ref());

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
