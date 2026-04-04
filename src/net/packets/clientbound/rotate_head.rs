use byteorder::{WriteBytesExt, BigEndian};
use crate::net::codec::write_var;

pub async fn send_rotate_head<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W, entity_id: i32, head_yaw: f32,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::ROTATE_HEAD as u8];

    write_var(&mut packet_data, entity_id)?;

    packet_data.write_u8((head_yaw / 360.0 * 256.0) as u8)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}