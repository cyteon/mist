use tokio::io::AsyncReadExt;

use crate::net::codec::read_var;

pub enum ClientPacket {
    Handshake,
    Ping,
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