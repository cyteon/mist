use byteorder::{BigEndian, WriteBytesExt};

use crate::net::codec::write_var;

pub async fn send_hurt_animation<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    entity_id: i32,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::HURT_ANIMATION as u8];

    write_var(&mut packet_data, entity_id)?;
    packet_data.write_f32::<BigEndian>(0.0)?; // relative yaw, idk we will do this later

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
