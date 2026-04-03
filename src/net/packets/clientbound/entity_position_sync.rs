use byteorder::{WriteBytesExt, BigEndian};
use crate::net::codec::write_var;

pub async fn send_entity_position_sync<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W, entity: &crate::types::entity::Entity
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::ENTITY_POSITION_SYNC as u8];

    write_var(&mut packet_data, entity.id)?;

    packet_data.write_f64::<BigEndian>(entity.x)?;
    packet_data.write_f64::<BigEndian>(entity.y)?;
    packet_data.write_f64::<BigEndian>(entity.z)?;

    packet_data.write_f64::<BigEndian>(entity.vx)?;
    packet_data.write_f64::<BigEndian>(entity.vy)?;
    packet_data.write_f64::<BigEndian>(entity.vz)?;

    packet_data.write_f32::<BigEndian>(entity.yaw)?;
    packet_data.write_f32::<BigEndian>(entity.pitch)?;

    packet_data.push(entity.on_ground as u8);

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}