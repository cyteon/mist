use crate::net::codec::write_var;

pub async fn sent_set_entity_data<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, entity: &crate::types::entity::Entity) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::SET_ENTITY_DATA as u8];

    write_var(&mut packet_data, entity.id)?;
    
    if let crate::types::entity::EntityType::Item(item, ..) = &entity.entity_type {
        packet_data.push(0x08); // item index
        write_var(&mut packet_data, 7)?; // item data

        write_var(&mut packet_data, item.count as i32)?;
        write_var(&mut packet_data, item.item_id)?;

        packet_data.push(0x00);
        packet_data.push(0x00);
    }

    packet_data.push(0xFF);

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}