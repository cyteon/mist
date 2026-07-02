pub use packets::*;

#[allow(unused)]
mod packets {
    include!(concat!(env!("OUT_DIR"), "/packets.rs"));
}

use std::io::Write;
use tokio::io::AsyncReadExt;

use crate::net::codec::{read_var, write_var};

pub enum ClientPacket {
    // status state
    Ping, // 0x01 in status

    // config state
    AcknowledgeFinishConfiguration,       // 0x03 in configuration
    KnownPacks(std::io::Cursor<Vec<u8>>), // 0x07 in configuration

    // play state
    ConfirmTeleprortion(std::io::Cursor<Vec<u8>>), // 0x00 in play
    ChatCommand(std::io::Cursor<Vec<u8>>),         // 0x06 in play
    ChatMessage(std::io::Cursor<Vec<u8>>),         // 0x08 in play
    ClientStatus(std::io::Cursor<Vec<u8>>),        // 0x0B in play
    ContainerClick(std::io::Cursor<Vec<u8>>),      // 0x11 in play
    ContainerClose(std::io::Cursor<Vec<u8>>),      // 0x12 in play
    PlayerAbilities(std::io::Cursor<Vec<u8>>),     // 0x27 in play
    PlayerAction(std::io::Cursor<Vec<u8>>),        // 0x28 in play
    SetCarriedItem(std::io::Cursor<Vec<u8>>),      // 0x34 in play
    SetCreativeModeSlot(std::io::Cursor<Vec<u8>>), // 0x37 in play
    UseItemOn(std::io::Cursor<Vec<u8>>),           // 0x3F in play
    UseItem(std::io::Cursor<Vec<u8>>),             // 0x40 in play
    SetPlayerPosition(std::io::Cursor<Vec<u8>>),   // 0x1D in play
    SetPlayerPositionAndRotation(std::io::Cursor<Vec<u8>>), // 0x1E in play
    PlayerInput(std::io::Cursor<Vec<u8>>),         // 0x2A in play
    SetPlayerRotation(std::io::Cursor<Vec<u8>>),   // 0x1F in play
    SwingArm(std::io::Cursor<Vec<u8>>),            // 0x3C in play
}

#[allow(unused)]
pub enum ProtocolState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

pub fn encode_packet(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();

    if data.len() > 256 {
        let uncompressed_len = data.len();

        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&data).unwrap();

        let compressed = encoder.finish().unwrap();

        let mut inner = Vec::with_capacity(5 + compressed.len());
        write_var(&mut inner, uncompressed_len as i32).unwrap();
        inner.extend_from_slice(&compressed);

        write_var(&mut result, inner.len() as i32).unwrap();
        result.extend_from_slice(&inner);
    } else {
        let mut inner = Vec::with_capacity(5);
        write_var(&mut inner, 0).unwrap();
        inner.extend_from_slice(&data);

        write_var(&mut result, inner.len() as i32).unwrap();
        result.extend_from_slice(&inner);
    }

    result
}

pub async fn read_packet<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    state: &ProtocolState,
    compression: bool,
) -> anyhow::Result<Option<ClientPacket>> {
    let packet_len = read_var(stream)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read packet length: {}", e))?;

    let mut packet_buf = vec![0u8; packet_len as usize];
    stream
        .read_exact(&mut packet_buf)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read packet data: {}", e))?;

    let mut cursor = std::io::Cursor::new(packet_buf);

    if compression {
        let data_len = read_var(&mut cursor)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read data length: {}", e))?;

        if data_len != 0 {
            let mut compressed = Vec::new();
            std::io::Read::read_to_end(&mut cursor, &mut compressed)
                .map_err(|e| anyhow::anyhow!("Failed to read compressed data: {}", e))?;

            let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
            let mut decompressed = Vec::with_capacity(data_len as usize);
            std::io::Read::read_to_end(&mut decoder, &mut decompressed)
                .map_err(|e| anyhow::anyhow!("Failed to decompress data: {}", e))?;

            cursor = std::io::Cursor::new(decompressed);
        }
    }

    let packet_id = read_var(&mut cursor)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read packet ID: {}", e))?;

    if packet_id != 0x0C && packet_id != 0x1D {
        // these packets are spammy
        crate::log::log(
            fancy_log::LogLevel::Debug,
            format!(
                "Received packet with ID: 0x{:02X} with length: {}",
                packet_id, packet_len
            )
            .as_str(),
        );
    }

    match state {
        ProtocolState::Status => match packet_id {
            status::serverbound::PING_REQUEST => Ok(Some(ClientPacket::Ping)),

            _ => Ok(None),
        },

        ProtocolState::Configuration => match packet_id {
            configuration::serverbound::FINISH_CONFIGURATION => {
                Ok(Some(ClientPacket::AcknowledgeFinishConfiguration))
            }

            configuration::serverbound::SELECT_KNOWN_PACKS => {
                Ok(Some(ClientPacket::KnownPacks(cursor)))
            }

            _ => Ok(None),
        },

        ProtocolState::Play => match packet_id {
            play::serverbound::ACCEPT_TELEPORTATION => {
                Ok(Some(ClientPacket::ConfirmTeleprortion(cursor)))
            }

            play::serverbound::CHAT_COMMAND => Ok(Some(ClientPacket::ChatCommand(cursor))),

            play::serverbound::CHAT => Ok(Some(ClientPacket::ChatMessage(cursor))),

            play::serverbound::PLAYER_ACTION => Ok(Some(ClientPacket::PlayerAction(cursor))),

            play::serverbound::USE_ITEM_ON => Ok(Some(ClientPacket::UseItemOn(cursor))),

            play::serverbound::USE_ITEM => Ok(Some(ClientPacket::UseItem(cursor))),

            play::serverbound::MOVE_PLAYER_POS => Ok(Some(ClientPacket::SetPlayerPosition(cursor))),

            play::serverbound::MOVE_PLAYER_POS_ROT => {
                Ok(Some(ClientPacket::SetPlayerPositionAndRotation(cursor)))
            }

            play::serverbound::PLAYER_INPUT => Ok(Some(ClientPacket::PlayerInput(cursor))),

            play::serverbound::MOVE_PLAYER_ROT => Ok(Some(ClientPacket::SetPlayerRotation(cursor))),

            play::serverbound::SET_CARRIED_ITEM => Ok(Some(ClientPacket::SetCarriedItem(cursor))),

            play::serverbound::SET_CREATIVE_MODE_SLOT => {
                Ok(Some(ClientPacket::SetCreativeModeSlot(cursor)))
            }

            play::serverbound::PLAYER_ABILITIES => Ok(Some(ClientPacket::PlayerAbilities(cursor))),

            play::serverbound::CLIENT_COMMAND => Ok(Some(ClientPacket::ClientStatus(cursor))),

            play::serverbound::SWING => Ok(Some(ClientPacket::SwingArm(cursor))),

            play::serverbound::CONTAINER_CLICK => Ok(Some(ClientPacket::ContainerClick(cursor))),

            play::serverbound::CONTAINER_CLOSE => Ok(Some(ClientPacket::ContainerClose(cursor))),

            _ => Ok(None),
        },

        _ => Ok(None),
    }
}
