use byteorder::{BigEndian, WriteBytesExt};

use crate::{net::codec::write_var, types::entity::EntityType};

pub async fn send_set_entity_data<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    entity: &crate::types::entity::Entity,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::SET_ENTITY_DATA as u8];

    write_var(&mut packet_data, entity.id)?;

    match &entity.entity_type {
        EntityType::Item(item_entity) => {
            packet_data.push(8); // item index
            write_var(&mut packet_data, 7)?; // item data

            write_var(&mut packet_data, item_entity.item_stack.count as i32)?;
            write_var(&mut packet_data, item_entity.item_stack.item_id as i32)?;

            packet_data.push(0x00);
            packet_data.push(0x00);
        }

        EntityType::Player(player) => {
            packet_data.push(9); // health index
            write_var(&mut packet_data, 3)?; // float

            packet_data.write_f32::<BigEndian>(player.health)?;
        }
    }

    packet_data.push(0xFF);

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
