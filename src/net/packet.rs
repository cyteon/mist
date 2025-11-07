use tokio::io::AsyncReadExt;

use crate::net::codec::{read_var, write_var};

pub enum ClientPacket {
    Handshake,
    Ping,
}

pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32,
}

pub enum ProtocolState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play
}

pub async fn read_packet<R: AsyncReadExt + Unpin>(stream: &mut R, state: &ProtocolState) -> anyhow::Result<Option<ClientPacket>> {
    let _packet_len = read_var(stream).await?;
    let packet_id = read_var(stream).await?;

    dbg!(packet_id);

    match state {
        ProtocolState::Status => {
            match packet_id {
                0x01 | 0x21 => {
                    Ok(Some(ClientPacket::Ping))
                },
                
                _ => {
                    Ok(None)
                }
            }
        },

        _ => {
            Ok(None)
        }
    }
}

pub async fn read_handshake<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<HandshakePacket> {
    let _packet_len = read_var(stream).await?;
    let packet_id = read_var(stream).await?;

    if packet_id != 0x00 {
        anyhow::bail!("Expected handshake packet ID 0x00, got {}", packet_id);
    }
    
    let protocol_version = read_var(stream).await? as i32;

    let addr_len = read_var(stream).await? as usize;
    let mut addr_buf = vec![0u8; addr_len];
    stream.read_exact(&mut addr_buf).await?;
    let server_address = String::from_utf8(addr_buf)?;

    let server_port = stream.read_u16().await?;
    let next_state = read_var(stream).await? as i32;

    Ok(HandshakePacket {
        protocol_version,
        server_address,
        server_port,
        next_state,
    })
}

pub async fn send_status_response<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let json = format!(r#"{{
        "version": {{
            "name": "1.21.10",
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

    let mut packet_data = vec![];

    write_var(&mut packet_data, 0x00).await?;
    write_var(&mut packet_data, json.len() as i32).await?;
    packet_data.extend_from_slice(json.as_bytes());

    write_var(stream, packet_data.len() as i32).await?;
    stream.write_all(&packet_data).await?;

    Ok(())
}