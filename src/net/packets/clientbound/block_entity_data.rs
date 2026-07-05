use crate::{
    net::codec::{write_position, write_var},
    types::block_entities::BlockEntityData,
};

pub async fn send_block_entity_data<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    position: (i32, i32, i32),
    block_entity: BlockEntityData,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::BLOCK_ENTITY_DATA as u8];

    write_position(&mut packet_data, position.0, position.1, position.2)?;

    write_var(&mut packet_data, block_entity.type_id())?;
    block_entity.write_nbt(&mut packet_data)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
