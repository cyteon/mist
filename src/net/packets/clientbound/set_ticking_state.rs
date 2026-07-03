use byteorder::{BigEndian, WriteBytesExt};

pub async fn send_set_ticking_state<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::TICKING_STATE as u8];

    packet_data.write_f32::<BigEndian>(20.0)?;
    packet_data.push(0u8); // not frozen

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
