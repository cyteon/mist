use byteorder::{BigEndian, WriteBytesExt};

use crate::{net::codec::write_var, types::player::Player};

pub async fn send_set_health<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    player: &Player,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::SET_HEALTH as u8];

    packet_data.write_f32::<BigEndian>(player.health)?;
    write_var(&mut packet_data, player.hunger as i32)?;
    packet_data.write_f32::<BigEndian>(player.saturation)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
