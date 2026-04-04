use byteorder::{WriteBytesExt, BigEndian};
use crate::net::codec::write_var;

pub async fn send_move_entity_rot<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W, entity: &crate::types::entity::Entity
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::MOVE_ENTITY_ROT as u8];

    write_var(&mut packet_data, entity.id)?;

    packet_data.write_u8((entity.yaw / 360.0 * 256.0) as u8)?;
    packet_data.write_u8((entity.pitch / 360.0 * 256.0) as u8)?;

    packet_data.push(entity.on_ground as u8);

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}