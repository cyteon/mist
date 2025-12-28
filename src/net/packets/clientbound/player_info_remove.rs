use crate::net::codec::write_var;

pub async fn send_player_info_remove<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![0x43];

    // todo

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}