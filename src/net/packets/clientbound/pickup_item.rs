use crate::net::codec::write_var;

pub async fn send_pickup_item<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    collected: i32,
    collector: i32,
    count: i32,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::TAKE_ITEM_ENTITY as u8];

    write_var(&mut packet_data, collected)?;
    write_var(&mut packet_data, collector)?;
    write_var(&mut packet_data, count)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
