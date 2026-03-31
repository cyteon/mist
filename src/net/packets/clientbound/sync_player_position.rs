use byteorder::{BigEndian, WriteBytesExt};

use crate::{net::codec::write_var, types::player::Player};

pub async fn send_sync_player_position<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, player: &Player) -> anyhow::Result<()> {    
    let mut packet_data = vec![crate::net::packet::play::clientbound::PLAYER_POSITION as u8];

    write_var(&mut packet_data, 1)?; // teleport id
    
    packet_data.write_f64::<BigEndian>(player.x)?;
    packet_data.write_f64::<BigEndian>(player.y)?;
    packet_data.write_f64::<BigEndian>(player.z)?;

    packet_data.write_f64::<BigEndian>(player.vx)?;
    packet_data.write_f64::<BigEndian>(player.vy)?;
    packet_data.write_f64::<BigEndian>(player.vz)?;

    packet_data.write_f32::<BigEndian>(player.yaw)?;
    packet_data.write_f32::<BigEndian>(player.pitch)?;

    packet_data.write_i32::<BigEndian>(0)?; // teleport flags

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}