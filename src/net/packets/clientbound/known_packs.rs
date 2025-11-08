use crate::net::codec::write_var;

pub async fn send_known_packs<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![];
    write_var(&mut packet_data, 0x0E).await?;

    write_var(&mut packet_data, 1).await?;

    write_var(&mut packet_data, "minecrafta".len() as i32).await?;
    packet_data.extend_from_slice("minecrafta".as_bytes());

    write_var(&mut packet_data, "core".len() as i32).await?;
    packet_data.extend_from_slice("core".as_bytes());

    write_var(&mut packet_data, "1.21".len() as i32).await?;
    packet_data.extend_from_slice("1.21".as_bytes());

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}