use crate::net::codec::write_var;

pub enum State {
    OnFire = 0x01,
    Sneaking = 0x02,
    Sprinting = 0x08,
    Swimming = 0x10,
    Invisible = 0x20,
    Glowing = 0x40,
    ElytraFlying = 0x80,
}

// todo: add rest of them
pub enum Pose {
    Standing = 0,
    Sneaking = 5,
}

pub async fn send_set_pose<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    entity_id: i32,
    states: Vec<State>,
    pose: Pose,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::SET_ENTITY_DATA as u8];

    write_var(&mut packet_data, entity_id)?;

    let mut state = 0;

    for s in states {
        state |= s as u8;
    }

    packet_data.push(0); // state index
    write_var(&mut packet_data, 0)?; // state data

    packet_data.push(state);

    packet_data.push(6); // pose index
    write_var(&mut packet_data, 20)?; // pose data
    write_var(&mut packet_data, pose as i32)?;

    packet_data.push(0xFF);

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
