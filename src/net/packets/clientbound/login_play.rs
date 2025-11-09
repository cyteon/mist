use crate::{config::SERVER_CONFIG, net::codec::write_var};

pub async fn send_login_play<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![0x2B];

    write_var(&mut packet_data, 1).await?; // palceholder for entity id
    write_var(&mut packet_data, false as i32).await?; // is hardcore
    write_var(&mut packet_data, 1).await?; // dimension count
    
    write_var(&mut packet_data, "world".len() as i32).await?;
    packet_data.extend_from_slice("world".as_bytes()); // dimension identifier

    write_var(&mut packet_data, SERVER_CONFIG.max_players as i32).await?;
    write_var(&mut packet_data, 16 as i32).await?; // view distance
    write_var(&mut packet_data, 8 as i32).await?; // simulation distance
    write_var(&mut packet_data, false as i32).await?; // reduced debug info
    write_var(&mut packet_data, true as i32).await?; // enable respawn screen
    write_var(&mut packet_data, false as i32).await?; // do limited crafting

    write_var(&mut packet_data, 1 as i32).await?; // dimension type
    write_var(&mut packet_data, "minecraft:overworld".len() as i32).await?;
    packet_data.extend_from_slice("minecraft:overworld".as_bytes());

    packet_data.extend_from_slice(&[0u8; 8]); // placeholder for first 8 bytes of hashed seed
    packet_data.extend_from_slice(&[1u8]); // placeholder for gamemode
    packet_data.extend_from_slice(&[1u8]); // placeholder for previous gamemode

    write_var(&mut packet_data, false as i32).await?; // is debug
    write_var(&mut packet_data, false as i32).await?; // is flat world
    write_var(&mut packet_data, false as i32).await?; // has death location
    write_var(&mut packet_data, 0 as i32).await?; // portal cooldown in ticks
    write_var(&mut packet_data, false as i32).await?; // enforce secure chat

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}