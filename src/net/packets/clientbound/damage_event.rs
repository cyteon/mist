use crate::net::codec::write_var;

pub async fn send_damage_event<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    entity_id: i32,
    source_type_id: i32,
    source_cause_id: i32,
    source_direct_id: i32,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::DAMAGE_EVENT as u8];

    write_var(&mut packet_data, entity_id)?;
    write_var(&mut packet_data, source_type_id)?;
    write_var(&mut packet_data, source_cause_id)?;
    write_var(&mut packet_data, source_direct_id)?;

    packet_data.push(0x00); // source pos

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
