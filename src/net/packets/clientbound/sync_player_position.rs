use tokio::io::AsyncWriteExt;

use crate::{net::codec::write_var, types::player::Player};

pub async fn send_sync_player_position<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, player: &Player) -> anyhow::Result<()> {    
    let mut packet_data = vec![0x41];

    write_var(&mut packet_data, 1).await?; // teleport id
    
    packet_data.write_f64(player.x).await?;
    packet_data.write_f64(player.y).await?;
    packet_data.write_f64(player.z).await?;

    packet_data.write_f64(player.vx).await?;
    packet_data.write_f64(player.vy).await?;
    packet_data.write_f64(player.vz).await?;

    packet_data.write_f32(player.yaw).await?;
    packet_data.write_f32(player.pitch).await?;

    packet_data.write_i32(0).await?; // teleport flags

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}