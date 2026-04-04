use crate::net::codec::write_var;

pub enum Animation {
    SwingMainArm = 0,
    // LeaveBed = 2,
    SwingOffArm = 3,
    // CriticalEffect = 4,
    // MagicCriticalEffect = 5,
}

pub async fn send_animate<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    entity_id: i32,
    animation: Animation,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::ANIMATE as u8];

    write_var(&mut packet_data, entity_id)?;
    packet_data.push(animation as u8);

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
