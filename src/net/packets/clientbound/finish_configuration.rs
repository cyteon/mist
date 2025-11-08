use crate::net::codec::write_var;

pub async fn send_finish_configuration<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![];
    write_var(&mut packet_data, 0x03).await?;

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}