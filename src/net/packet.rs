use fancy_log::log;
use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub enum ClientPacket {
    Handshake,
    Ping,
    LoginStart,
    EncryptionResponse,
    KnownPacks,
    AcknowledgeFinishConfiguration
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
    let packet_id = read_var(stream).await?;

    log(
        fancy_log::LogLevel::Debug, 
        format!("Received packet with ID: 0x{:02X} with length: {}", packet_id, packet_len).as_str()
    );

    match state {
        ProtocolState::Status => {
            match packet_id {
                0x01 => {
                    Ok(Some(ClientPacket::Ping))
                },
                
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
                // According to the protocol docs, this is supposed to be 0x03
                // But every time after sending the finish config, i get a 0x01 back
                // Which makes no sense as 0x01 in the config stage is a cookie request response
                // But for now ill just parse it as an acknowledgment, unless shit breaks
                // TODO: fix
                0x01 => {
                    Ok(Some(ClientPacket::AcknowledgeFinishConfiguration))
                },

                0x07 => {
                    Ok(Some(ClientPacket::KnownPacks))
                },
                
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

        _ => {
            if packet_len > 1 {
                for _ in 0..(packet_len - 1) {
                    let _ = stream.read_u8().await?;
                }
            }

            Ok(None)
        }
    }
}