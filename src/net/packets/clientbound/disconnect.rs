use crate::net::codec::write_var;

pub async fn send_disconnect_login<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, reason: &str) -> anyhow::Result<()> {
    let json = format!(r#"{{
        "text": "{}"
    }}"#, reason);

    let mut packet_data = vec![];
    write_var(&mut packet_data, 0x00).await?;

    write_var(&mut packet_data, json.len() as i32).await?;
    packet_data.extend_from_slice(json.as_bytes());

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}