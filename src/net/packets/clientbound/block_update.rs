use fancy_log::LogLevel;

use crate::{
    log,
    net::codec::{write_position, write_var},
};

pub async fn broadcast_block_update(x: i32, y: i32, z: i32, block_id: i32) -> anyhow::Result<()> {
    let player_positions = crate::types::player::PLAYER_POSITIONS.read().await;
    let view_distance_blocks = crate::config::SERVER_CONFIG.view_distance as i32 * 16;

    for (uuid, pos) in player_positions.iter() {
        let distance_squared = (pos.0 - x as f64).powi(2) + (pos.2 - z as f64).powi(2);

        if distance_squared < (view_distance_blocks as f64).powi(2) {
            let tx = match crate::server::conn::PLAYER_SOCKET_MAP
                .read()
                .await
                .get(uuid)
                .cloned()
            {
                Some(tx) => tx,
                None => {
                    log::log(
                        LogLevel::Warn,
                        &format!(
                            "Player {} not found in socket map, possibly disconnected?",
                            uuid
                        ),
                    );
                    continue;
                }
            };

            let mut buffer = Vec::new();
            send_block_update(&mut buffer, x, y, z, block_id).await?;
            let _ = tx.send(buffer);
        }
    }

    Ok(())
}

pub async fn send_block_update<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    x: i32,
    y: i32,
    z: i32,
    block_id: i32,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::BLOCK_UPDATE as u8];

    write_position(&mut packet_data, x, y, z)?;
    write_var(&mut packet_data, block_id)?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
