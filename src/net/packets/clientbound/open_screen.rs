use crate::net::codec::write_var;

pub enum WindowType {
    Chest = 2,
    BlastFurnace = 10,
    CraftingTable = 12,
    Furnace = 14,
    Smoker = 22,
}

pub async fn send_open_screen<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    window_id: i32,
    window_type: WindowType,
    window_title: &str,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::OPEN_SCREEN as u8];

    write_var(&mut packet_data, window_id)?;
    write_var(&mut packet_data, window_type as i32)?;

    craftflow_nbt::to_writer(
        &mut packet_data,
        &craftflow_nbt::DynNBT::String(window_title.to_string()),
    )?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
