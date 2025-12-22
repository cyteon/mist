use crate::net::codec::write_var;
use tokio::io::AsyncWriteExt;

pub async fn send_chunk_data_with_light<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    chunk_x: i32,
    chunk_z: i32,
) -> anyhow::Result<()> {
    let mut packet_data = vec![0x2C];

    packet_data.write_i32(chunk_x).await?;
    packet_data.write_i32(chunk_z).await?;

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}