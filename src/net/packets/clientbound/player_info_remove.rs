use crate::net::codec::write_var;

pub async fn send_player_info_remove<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, uuids: &[String]) -> anyhow::Result<()> {
    let mut packet_data = vec![0x43];

    write_var(&mut packet_data, uuids.len() as i32)?;
    for uuid in uuids {
        let uuid_clean = uuid.replace("-", "");
        let uuid_vec = hex::decode(uuid_clean)?;
        packet_data.extend_from_slice(&uuid_vec);
    }

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}