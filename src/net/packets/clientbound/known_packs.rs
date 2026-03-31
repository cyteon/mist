use crate::net::codec::write_var;

pub async fn send_known_packs<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::configuration::clientbound::SELECT_KNOWN_PACKS as u8];

    write_var(&mut packet_data, 1)?;

    write_var(&mut packet_data, "minecrafta".len() as i32)?;
    packet_data.extend_from_slice("minecrafta".as_bytes());

    write_var(&mut packet_data, "core".len() as i32)?;
    packet_data.extend_from_slice("core".as_bytes());

    write_var(&mut packet_data, "1.21".len() as i32)?;
    packet_data.extend_from_slice("1.21".as_bytes());

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}