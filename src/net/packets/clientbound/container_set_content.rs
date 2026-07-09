use crate::{
    net::codec::{write_slot, write_var},
    types::items::ItemStack,
};

pub async fn send_container_set_content<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    window_id: u8,
    inventory: Vec<Option<ItemStack>>,
    carried_item: Option<ItemStack>,
    state_id: i32,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::CONTAINER_SET_CONTENT as u8];

    write_var(&mut packet_data, window_id as i32)?;
    write_var(&mut packet_data, state_id)?;

    write_var(&mut packet_data, inventory.len() as i32)?;
    for item in inventory.iter() {
        write_slot(&mut packet_data, *item)?;
    }

    if let Some(_) = carried_item {
        write_slot(&mut packet_data, carried_item)?;
    } else {
        write_var(&mut packet_data, 0)?;
    }

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
