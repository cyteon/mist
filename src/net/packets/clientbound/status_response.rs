use crate::net::codec::{write_string, write_var};

pub async fn send_status_response<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
) -> anyhow::Result<()> {
    let json = format!(
        r#"{{
        "version": {{
            "name": "{}",
            "protocol": {}
        }},
        "players": {{
            "max": {},
            "online": {}
        }},
        "description": {{
            "text": "{}"
        }}
    }}"#,
        crate::SERVER_VERSION,
        crate::SERVER_PROTOCOL_VERSION,
        crate::config::SERVER_CONFIG.max_players,
        crate::server::state::play::PLAYERS.read().await.len(),
        crate::config::SERVER_CONFIG.motd
    );

    let mut packet_data = vec![crate::net::packet::status::clientbound::STATUS_RESPONSE as u8];

    write_string(&mut packet_data, &json)?;

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
