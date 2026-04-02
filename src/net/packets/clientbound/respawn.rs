use byteorder::WriteBytesExt;

use crate::net::codec::{write_var, write_string};

pub async fn send_respawn<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, player: &crate::types::player::Player) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::RESPAWN as u8];

    write_var(&mut packet_data, 0)?; // dimension type
    write_string(&mut packet_data, "overworld")?; // dimension identifier

    packet_data.extend_from_slice(&[1u8; 8]); // placeholder for first 8 bytes of hashed seed
    packet_data.push(player.gamemode as u8);
    packet_data.write_i8(-1)?; // previous gamemode, -1 = undefined

    packet_data.push(false as u8); // is debug
    packet_data.push(false as u8); // is flat world
    packet_data.push(false as u8); // has death location

    write_var(&mut packet_data, 0)?; // portal cooldown in ticks
    write_var(&mut packet_data, 62)?; // sea level

    packet_data.push(false as u8); // data kept

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}