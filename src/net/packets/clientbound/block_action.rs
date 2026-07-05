use crate::net::codec::{write_position, write_var};

pub async fn send_block_action<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    position: (i32, i32, i32),
    action_id: u8,
    action_param: u8,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::BLOCK_EVENT as u8];

    write_position(&mut packet_data, position.0, position.1, position.2)?;

    packet_data.push(action_id);
    packet_data.push(action_param);

    write_var(&mut packet_data, 0)?; // block type, vanilla client shouldnt need this?

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
