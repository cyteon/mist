use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub enum ClientPacket {
    Handshake,

    // login state
    LoginStart, // 0x00
    EncryptionResponse, // 0x01

    // status state
    Ping, // 0x01 in status

    // config state
    AcknowledgeFinishConfiguration, // 0x03 in configuration
    KnownPacks(std::io::Cursor<Vec<u8>>), // 0x07 in configuration

    // play state
    ConfirmTeleprortion(std::io::Cursor<Vec<u8>>), // 0x00 in play
    ChatMessage(std::io::Cursor<Vec<u8>>), // 0x08 in play
    PlayerAction(std::io::Cursor<Vec<u8>>), // 0x28 in play
    UseItemOn(std::io::Cursor<Vec<u8>>), // 0x3F in play
    SetPlayerPositionAndRotation(std::io::Cursor<Vec<u8>>), // 0x1E in play
}

pub enum ProtocolState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play
}

pub async fn read_packet<R: AsyncReadExt + Unpin>(stream: &mut R, state: &ProtocolState) -> anyhow::Result<Option<ClientPacket>> {
    let packet_len = read_var(stream).await?;

    let mut packet_buf = vec![0u8; packet_len as usize];
    stream.read_exact(&mut packet_buf).await?;

    let mut cursor = std::io::Cursor::new(packet_buf);
    let packet_id = read_var(&mut cursor).await?;

    if packet_id != 0x0C && packet_id != 0x1D { // these packets are spammy
        crate::log::log(
            fancy_log::LogLevel::Debug, 
            format!("Received packet with ID: 0x{:02X} with length: {}", packet_id, packet_len).as_str()
        );
    }

    match state {
        ProtocolState::Status => {
            match packet_id {
                0x01 => {
                    Ok(Some(ClientPacket::Ping))
                },
                
                _ => Ok(None)
            }
        },

        ProtocolState::Configuration => {
            match packet_id {
                0x03 => {
                    Ok(Some(ClientPacket::AcknowledgeFinishConfiguration))
                },

                0x07 => {
                    Ok(Some(ClientPacket::KnownPacks(cursor)))
                },
                
                _ => {
                    Ok(None)
                }
            }
        },

        ProtocolState::Play => {
            match packet_id {
                0x00 => {
                    Ok(Some(ClientPacket::ConfirmTeleprortion(cursor)))
                },

                0x08 => {
                    Ok(Some(ClientPacket::ChatMessage(cursor)))
                },

                0x28 => {
                    Ok(Some(ClientPacket::PlayerAction(cursor)))
                },

                0x3F => {
                    Ok(Some(ClientPacket::UseItemOn(cursor)))
                },

                0x1E => {
                    Ok(Some(ClientPacket::SetPlayerPositionAndRotation(cursor)))
                },
                
                _ => Ok(None)
            }
        },

        _ => {
            Ok(None)
        }
    }
}