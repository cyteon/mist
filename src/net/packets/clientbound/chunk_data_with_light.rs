use crate::net::codec::write_var;
use crate::world::chunks::Chunk;
use byteorder::{WriteBytesExt, BigEndian};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light
pub async fn send_chunk_data_with_light<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    chunk: &Chunk,
) -> anyhow::Result<()> {
    let mut packet_data = vec![0x2C];
    
    packet_data.write_i32::<BigEndian>(chunk.x)?;
    packet_data.write_i32::<BigEndian>(chunk.z)?;
    
    // heightmap
    write_var(&mut packet_data, 0)?;
    
    let mut data_section = Vec::new();
    for section in &chunk.sections {
        let block_count = section.block_count();
        data_section.write_i16::<BigEndian>(block_count)?;
        
        section.blocks.write_paletted_container(&mut data_section)?;
        
        data_section.write_u8(0)?; // 0 bpe
        data_section.write_u8(1)?; // plains biome
    }
    
    write_var(&mut packet_data, data_section.len() as i32)?;
    packet_data.extend_from_slice(&data_section);
    
    // block entities
    write_var(&mut packet_data, 0)?;

    let section_count = chunk.sections.len() + 2;
    let mask = (1u64 << section_count) - 1;

    // light data - todo: make it dynamic and not just fullbright

    // sky light mask
    write_var(&mut packet_data, 1)?;
    packet_data.write_i64::<BigEndian>(mask as i64)?;

    // block Light Mask
    write_var(&mut packet_data, 1)?;
    packet_data.write_i64::<BigEndian>(mask as i64)?;

    // empty sky light mask
    write_var(&mut packet_data, 0)?;

    // empty block light mask
    write_var(&mut packet_data, 0)?;

    // sky light array
    write_var(&mut packet_data, section_count as i32)?;
    for _ in 0..section_count {
        write_var(&mut packet_data, 2048)?;
        packet_data.extend_from_slice(&[0xFF; 2048]);
    }

    // block light arry
    write_var(&mut packet_data, section_count as i32)?;
    for _ in 0..section_count {
        write_var(&mut packet_data, 2048)?;
        packet_data.extend_from_slice(&[0xFF; 2048]);
    }
    
    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;
    
    Ok(())
}