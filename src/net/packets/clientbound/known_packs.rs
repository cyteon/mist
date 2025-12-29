use crate::net::codec::write_var;

pub async fn send_known_packs<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![0x0E];

    write_var(&mut packet_data, 1)?;

    write_var(&mut packet_data, "minecrafta".len() as i32)?;
    packet_data.extend_from_slice("minecrafta".as_bytes());

    write_var(&mut packet_data, "core".len() as i32)?;
    packet_data.extend_from_slice("core".as_bytes());

    write_var(&mut packet_data, "1.21".len() as i32)?;
    packet_data.extend_from_slice("1.21".as_bytes());

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}