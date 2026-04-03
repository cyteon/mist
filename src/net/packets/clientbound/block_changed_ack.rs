use crate::net::codec::write_var;

pub async fn send_block_changed_ack<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, seq: i32) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::BLOCK_CHANGED_ACK as u8];

    write_var(&mut packet_data, seq)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}