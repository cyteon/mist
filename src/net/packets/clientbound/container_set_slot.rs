use crate::net::codec::write_var;

pub async fn send_container_set_slot<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    window_id: u8,
    slot: i16,
    item_stack: Option<crate::types::items::ItemStack>
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::CONTAINER_SET_SLOT as u8];

    write_var(&mut packet_data, window_id as i32)?;
    write_var(&mut packet_data, 0)?; // state id, TODO: implement

    match item_stack {
        Some(item_stack) => {
            write_var(&mut packet_data, item_stack.count as i32)?;
            write_var(&mut packet_data, item_stack.item_id)?;

            packet_data.push(0x00);
            packet_data.push(0x00);
        },

        None => {
            write_var(&mut packet_data, 0)?; // empty item stack
        }
    }

    write_var(&mut packet_data, 0)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}