use crate::net::codec::{write_lpvec3, write_var};
use byteorder::{BigEndian, WriteBytesExt};

pub async fn send_spawn_entity<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    entity: &crate::types::entity::Entity,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::ADD_ENTITY as u8];

    write_var(&mut packet_data, entity.id)?;
    packet_data.write_u128::<BigEndian>(entity.uuid)?;

    match entity.entity_type {
        crate::types::entity::EntityType::Item(..) => write_var(&mut packet_data, 71)?,
        crate::types::entity::EntityType::Player(_) => write_var(&mut packet_data, 155)?,
    }

    packet_data.write_f64::<BigEndian>(entity.x)?;
    packet_data.write_f64::<BigEndian>(entity.y)?;
    packet_data.write_f64::<BigEndian>(entity.z)?;

    write_lpvec3(&mut packet_data, entity.vx, entity.vy, entity.vz)?;

    packet_data.write_i8((entity.pitch / 360.0 * 256.0) as i8)?;
    packet_data.write_i8((entity.yaw / 360.0 * 256.0) as i8)?;
    packet_data.write_i8(0)?; // head yaw

    write_var(&mut packet_data, 0)?; // data

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
