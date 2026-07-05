use crate::net::codec::write_var;
use crate::world::chunks::Chunk;
use byteorder::{BigEndian, WriteBytesExt};

const FULLBRIGHT_ENTRY: [u8; 2048] = [0xFFu8; 2048];

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light
pub async fn send_level_chunk_with_light<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    chunk: &Chunk,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::LEVEL_CHUNK_WITH_LIGHT as u8];

    packet_data.write_i32::<BigEndian>(chunk.x)?;
    packet_data.write_i32::<BigEndian>(chunk.z)?;

    // heightmap
    write_var(&mut packet_data, 0)?;

    let mut data_section = Vec::new();
    for section in &chunk.sections {
        data_section.write_i16::<BigEndian>(section.block_count)?;

        section.blocks.write_paletted_container(&mut data_section)?;

        data_section.write_u8(0)?; // 0 bpe
        data_section.write_u8(1)?;
    }

    write_var(&mut packet_data, data_section.len() as i32)?;
    packet_data.extend_from_slice(&data_section);

    // block entities
    write_var(&mut packet_data, chunk.block_entities.len() as i32)?;

    for block_entity in &chunk.block_entities {
        let cords = block_entity.0;

        let lx = (cords.0 & 15) as u8;
        let lz = (cords.2 & 15) as u8;
        let xz = (lx << 4) | lz;

        packet_data.write_u8(xz)?;
        packet_data.write_i16::<BigEndian>(cords.1 as i16)?;

        write_var(&mut packet_data, block_entity.1.type_id())?;

        block_entity.1.write_nbt(&mut packet_data)?;
    }

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
        packet_data.extend_from_slice(&FULLBRIGHT_ENTRY);
    }

    // block light arry
    write_var(&mut packet_data, section_count as i32)?;
    for _ in 0..section_count {
        write_var(&mut packet_data, 2048)?;
        packet_data.extend_from_slice(&FULLBRIGHT_ENTRY);
    }

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
