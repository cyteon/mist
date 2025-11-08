use crate::net::codec::write_var;

pub async fn send_login_success<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, username: &str, uuid: &str) -> anyhow::Result<()> {
    let mut packet_data = vec![];
    write_var(&mut packet_data, 0x02).await?;

    let uuid_clean = uuid.replace("-", "");
    let uuid_bytes = hex::decode(&uuid_clean)?;
    packet_data.extend_from_slice(&uuid_bytes);

    write_var(&mut packet_data, username.len() as i32).await?;
    packet_data.extend_from_slice(username.as_bytes());

    // 0 properties for now, todo: add the properties
    write_var(&mut packet_data, 0).await?;

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}