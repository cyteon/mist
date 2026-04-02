use byteorder::{WriteBytesExt, BigEndian};
use crate::net::codec::write_var;

pub async fn send_spawn_entity<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, entity: &mut crate::types::entity::Entity) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::ADD_ENTITY as u8];

    write_var(&mut packet_data, entity.id)?;
    packet_data.write_u128::<BigEndian>(entity.uuid)?;

    match entity.entity_type {
        crate::types::entity::EntityType::Item(item_stack) => write_var(&mut packet_data, 71)?,
        _ => write_var(&mut packet_data, 0)? // todo
    }

    packet_data.write_f64::<BigEndian>(entity.x)?;
    packet_data.write_f64::<BigEndian>(entity.y)?;
    packet_data.write_f64::<BigEndian>(entity.z)?;

    packet_data.write_u8(0u8)?;

    packet_data.write_i8((entity.pitch / 360.0 * 256.0) as i8)?;
    packet_data.write_i8((entity.yaw / 360.0 * 256.0) as i8)?;
    packet_data.write_i8(0)?; // head pitch

    write_var(&mut packet_data, 0)?; // data

    println!("sending {} bytes: {:?}", packet_data.len(), packet_data);

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}