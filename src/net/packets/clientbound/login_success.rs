use crate::net::codec::write_var;

pub async fn send_login_success<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, username: &str, uuid: &str) -> anyhow::Result<()> {
    let mut packet_data = vec![];
    write_var(&mut packet_data, 0x02).await?;
    
    write_var(&mut packet_data, uuid.len() as i32).await?;
    packet_data.extend_from_slice(uuid.as_bytes());

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}