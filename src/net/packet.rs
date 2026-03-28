include!(concat!(env!("OUT_DIR"), "/packets.rs"));

use tokio::io::AsyncReadExt;
use std::collections::HashMap;
use once_cell::sync::Lazy;

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
    PlayerInput(std::io::Cursor<Vec<u8>>), // 0x2A in play
    SetPlayerRotation(std::io::Cursor<Vec<u8>>), // 0x1F in play
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
                status::serverbound::PING_REQUEST => {
                    Ok(Some(ClientPacket::Ping))
                },
                
                _ => Ok(None)
            }
        },

        ProtocolState::Configuration => {
            match packet_id {
                configuration::serverbound::FINISH_CONFIGURATION => {
                    Ok(Some(ClientPacket::AcknowledgeFinishConfiguration))
                },

                configuration::serverbound::SELECT_KNOWN_PACKS => {
                    Ok(Some(ClientPacket::KnownPacks(cursor)))
                },
                
                _ => {
                    Ok(None)
                }
            }
        },

        ProtocolState::Play => {
            match packet_id {
                play::serverbound::ACCEPT_TELEPORTATION => {
                    Ok(Some(ClientPacket::ConfirmTeleprortion(cursor)))
                },

                play::serverbound::CHAT => {
                    Ok(Some(ClientPacket::ChatMessage(cursor)))
                },

                play::serverbound::PLAYER_ACTION => {
                    Ok(Some(ClientPacket::PlayerAction(cursor)))
                },

                play::serverbound::USE_ITEM_ON => {
                    Ok(Some(ClientPacket::UseItemOn(cursor)))
                },

                play::serverbound::MOVE_PLAYER_POS => {
                    Ok(Some(ClientPacket::SetPlayerPositionAndRotation(cursor)))
                },

                play::serverbound::PLAYER_INPUT => {
                    Ok(Some(ClientPacket::PlayerInput(cursor)))
                },

                play::serverbound::MOVE_PLAYER_ROT => {
                    Ok(Some(ClientPacket::SetPlayerRotation(cursor)))
                },
                
                _ => Ok(None)
            }
        },

        _ => {
            Ok(None)
        }
    }
}