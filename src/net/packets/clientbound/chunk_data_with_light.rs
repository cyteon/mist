use crate::net::codec::write_var;
use crate::world::chunks::Chunk;
use tokio::io::AsyncWriteExt;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light
pub async fn send_chunk_data_with_light<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    chunk: &Chunk,
) -> anyhow::Result<()> {
    let mut packet_data = vec![0x2C];
    
    packet_data.write_i32(chunk.x).await?;
    packet_data.write_i32(chunk.z).await?;
    
    // heightmap
    write_var(&mut packet_data, 0).await?;
    
    let mut data_section = Vec::new();
    for section in &chunk.sections {
        let block_count = section.block_count();
        data_section.write_i16(block_count).await?;
        
        section.blocks.write_paletted_container(&mut data_section).await?;
        
        data_section.write_u8(0).await?; // 0 bpe
        data_section.write_u8(1).await?; // plains biome
    }
    
    write_var(&mut packet_data, data_section.len() as i32).await?;
    packet_data.write_all(&data_section).await?;
    
    // block entities
    write_var(&mut packet_data, 0).await?;

    // light data - todo: actually make
    write_var(&mut packet_data, 0).await?; // sky light mask
    write_var(&mut packet_data, 0).await?; // block light mask
    write_var(&mut packet_data, 0).await?; // empty sky light mask
    write_var(&mut packet_data, 0).await?; // empty block light mask

    let sky_light: Vec<u8> = (0..chunk.sections.len() * 2048).map(|_| 0u8).collect();
    write_var(&mut packet_data, sky_light.len() as i32).await?;
    packet_data.write_all(&sky_light).await?;

    let block_light: Vec<u8> = (0..chunk.sections.len() * 2048).map(|_| 0u8).collect();
    write_var(&mut packet_data, block_light.len() as i32).await?;
    packet_data.write_all(&block_light).await?;

    //dbg!(&packet_data.len()); // okay so this made the code not break, maybe cause it adds slight io delay
    // slight timeout to simulate the io delay??
    tokio::time::sleep(std::time::Duration::from_micros(50)).await; // this works :O
    
    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;
    
    Ok(())
}