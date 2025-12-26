use tokio::io::AsyncWriteExt;

use crate::{config::SERVER_CONFIG, net::codec::write_var};

pub async fn send_login_play<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![0x30];

    packet_data.write_i32(1).await?; // palceholder for entity id
    packet_data.push(false as u8); // is hardcore

    write_var(&mut packet_data, 1).await?; // dimension count
    
    write_var(&mut packet_data, "overworld".len() as i32).await?;
    packet_data.extend_from_slice("overworld".as_bytes()); // dimension identifier

    write_var(&mut packet_data, SERVER_CONFIG.max_players as i32).await?;
    write_var(&mut packet_data, SERVER_CONFIG.view_distance as i32).await?;
    write_var(&mut packet_data, SERVER_CONFIG.simulation_distance as i32).await?;

    packet_data.push(false as u8); // reduced debug info
    packet_data.push(true as u8); // enable respawn screen
    packet_data.push(false as u8); // do limited crafting

    write_var(&mut packet_data, 0 as i32).await?; // dimension type
    write_var(&mut packet_data, "minecraft:overworld".len() as i32).await?;
    packet_data.extend_from_slice("minecraft:overworld".as_bytes());

    packet_data.extend_from_slice(&[1u8; 8]); // placeholder for first 8 bytes of hashed seed
    packet_data.push(1u8); // placeholder for gamemode
    packet_data.push(1u8); // placeholder for previous gamemode

    packet_data.push(false as u8); // is debug
    packet_data.push(false as u8); // is flat world
    packet_data.push(false as u8); // has death location
    write_var(&mut packet_data, 0).await?; // portal cooldown in ticks
    write_var(&mut packet_data, 62).await?; // sea level
    packet_data.push(false as u8); // enforce secure chat

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}