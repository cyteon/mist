use crate::net::codec::write_var;

pub async fn send_status_response<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let json = format!(r#"{{
        "version": {{
            "name": "1.21.9 - 1.21.10",
            "protocol": 773
        }},
        "players": {{
            "max": {},
            "online": 0
        }},
        "description": {{
            "text": "{}"
        }}
    }}"#,
        crate::config::SERVER_CONFIG.max_players,
        crate::config::SERVER_CONFIG.motd
    );

    let mut packet_data = vec![0x00];

    write_var(&mut packet_data, json.len() as i32).await?;
    packet_data.extend_from_slice(json.as_bytes());

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}