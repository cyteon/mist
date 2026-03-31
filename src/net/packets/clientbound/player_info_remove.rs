use crate::net::codec::write_var;

pub async fn send_player_info_remove<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W, uuids: Vec<&String>) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::PLAYER_INFO_REMOVE as u8];

    write_var(&mut packet_data, uuids.len() as i32)?;
    for uuid in uuids {
        let uuid_clean = uuid.replace("-", "");
        let uuid_vec = hex::decode(uuid_clean)?;
        packet_data.extend_from_slice(&uuid_vec);
    }

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}