use crate::net::codec::write_var;

pub async fn send_plugin_message<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![0x01];

    let channel = b"minecraft:brand";
    let chan_len = channel.len() as i32;

    write_var(&mut packet_data, chan_len)?;

    packet_data.extend_from_slice(channel);

    let message = b"mist";
    let msg_len = message.len() as i32;

    write_var(&mut packet_data, msg_len)?;
    packet_data.extend_from_slice(message);

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}