use crate::net::codec::write_var;

pub async fn send_set_compression<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::login::clientbound::LOGIN_COMPRESSION as u8];

    write_var(&mut packet_data, 256)?;

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}