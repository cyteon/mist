use fancy_log::log;
use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub enum ClientPacket {
    Handshake,
    Ping,
    LoginStart,
    EncryptionResponse,
    KnownPacks(std::io::Cursor<Vec<u8>>),
    AcknowledgeFinishConfiguration,
    ConfirmTeleprortion(std::io::Cursor<Vec<u8>>),
    PlayerAction(std::io::Cursor<Vec<u8>>),
    UseItemOn(std::io::Cursor<Vec<u8>>),
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
        log(
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
                
                _ => {
                    Ok(None)
                }
            }
        },

        ProtocolState::Login => {
            match packet_id {
                0x00 => {
                    Ok(Some(ClientPacket::LoginStart))
                },

                0x01 => {
                    Ok(Some(ClientPacket::EncryptionResponse))
                }
                
                _ => {
                    if packet_len > 1 {
                        for _ in 0..(packet_len - 1) {
                            let _ = stream.read_u8().await?;
                        }
                    }

                    Ok(None)
                }
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

                0x28 => {
                    Ok(Some(ClientPacket::PlayerAction(cursor)))
                },

                0x3F => {
                    Ok(Some(ClientPacket::UseItemOn(cursor)))
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