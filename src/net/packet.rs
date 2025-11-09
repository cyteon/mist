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
    let _packet_len = read_var(stream).await?;
    let packet_id = read_var(stream).await?;

    dbg!(&packet_id);

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
                    Ok(Some(ClientPacket::KnownPacks))
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