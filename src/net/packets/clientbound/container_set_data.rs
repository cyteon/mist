use byteorder::{BigEndian, WriteBytesExt};

use crate::net::codec::write_var;

pub async fn send_container_set_data<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    window_id: u8,
    property: i16,
    value: i16,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::CONTAINER_SET_DATA as u8];

    write_var(&mut packet_data, window_id as i32)?;

    packet_data.write_i16::<BigEndian>(property)?;
    packet_data.write_i16::<BigEndian>(value)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
