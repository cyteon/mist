use crate::net::codec::write_var;

pub async fn send_remove_entities<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, entities: Vec<i32>) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::REMOVE_ENTITIES as u8];

    write_var(&mut packet_data, entities.len() as i32)?;

    for entity_id in entities {
        write_var(&mut packet_data, entity_id)?;
    }

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}