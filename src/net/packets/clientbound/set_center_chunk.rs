use crate::net::codec::write_var;

pub async fn send_set_center_chunk<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, x: i32, z: i32) -> anyhow::Result<()> {
    let mut packet_data = vec![0x5C];

    write_var(&mut packet_data, x)?;
    write_var(&mut packet_data, z)?;

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}