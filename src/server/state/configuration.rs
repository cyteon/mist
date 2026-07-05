use std::time::Duration;

use crate::log::LogLevel;
use tokio::{io::AsyncWriteExt, net::TcpStream, time::timeout};

use crate::{
    net::{
        packet::{ClientPacket, ProtocolState, encode_packet, read_packet},
        packets::{
            clientbound::{
                finish_configuration::send_finish_configuration, known_packs::send_known_packs,
                login_play::send_login_play, plugin_message::send_plugin_message,
                registry_data::send_all_registers, update_tags::send_update_tags,
            },
            serverbound::known_packs::read_known_packs,
        },
    },
    server::{encryption::EncryptedStream, state::play},
    types::player::Player,
};

pub async fn configuration(
    mut socket: EncryptedStream<TcpStream>,
    player: Player,
) -> anyhow::Result<()> {
    crate::log::log(
        LogLevel::Debug,
        format!("{} has entered the configuration state", player.username).as_str(),
    );

    let mut buffer = Vec::new();
    send_plugin_message(&mut buffer).await?;

    let encoded = match encode_packet(&buffer) {
        Ok(packet) => packet,

        Err(e) => {
            crate::log::log(
                LogLevel::Error,
                format!(
                    "Failed to encode plugin message packet for {}: {}",
                    player.username, e
                )
                .as_str(),
            );

            socket.shutdown().await?;
            return Ok(());
        }
    };

    socket.write_all(&encoded).await?;

    let mut buffer = Vec::new();
    send_known_packs(&mut buffer).await?;

    let encoded = match encode_packet(&buffer) {
        Ok(packet) => packet,

        Err(e) => {
            crate::log::log(
                LogLevel::Error,
                format!(
                    "Failed to encode known packs packet for {}: {}",
                    player.username, e
                )
                .as_str(),
            );

            socket.shutdown().await?;
            return Ok(());
        }
    };

    socket.write_all(&encoded).await?;

    crate::log::log(
        LogLevel::Debug,
        format!("Sent known packs to {}", player.username).as_str(),
    );

    loop {
        match timeout(
            Duration::from_secs(15),
            read_packet(&mut socket, &ProtocolState::Configuration, true),
        )
        .await
        {
            Ok(Ok(Some(packet))) => match packet {
                ClientPacket::KnownPacks(mut cursor) => {
                    read_known_packs(&mut cursor).await?;
                    crate::log::log(
                        LogLevel::Debug,
                        format!("{} has sent known packs", player.username).as_str(),
                    );

                    send_all_registers(&mut socket).await?;
                    crate::log::log(
                        LogLevel::Debug,
                        format!("Sent registry data to {}", player.username).as_str(),
                    );

                    send_update_tags(&mut socket).await?;
                    crate::log::log(
                        LogLevel::Debug,
                        format!("Sent tags to {}", player.username).as_str(),
                    );

                    let mut buffer = Vec::new();
                    send_finish_configuration(&mut buffer).await?;

                    let encoded = match encode_packet(&buffer) {
                        Ok(packet) => packet,

                        Err(e) => {
                            crate::log::log(
                                LogLevel::Error,
                                format!(
                                    "Failed to encode finish configuration packet for {}: {}",
                                    player.username, e
                                )
                                .as_str(),
                            );
                            socket.shutdown().await?;
                            break;
                        }
                    };

                    socket.write_all(&encoded).await?;

                    crate::log::log(
                        LogLevel::Debug,
                        format!("Sent finish configuration to {}", player.username).as_str(),
                    );
                }

                ClientPacket::AcknowledgeFinishConfiguration => {
                    crate::log::log(
                        LogLevel::Debug,
                        format!("{} has finished configuration", player.username).as_str(),
                    );

                    let mut buffer = Vec::new();
                    send_login_play(&mut buffer, &player).await?;

                    let encoded = match encode_packet(&buffer) {
                        Ok(packet) => packet,

                        Err(e) => {
                            crate::log::log(
                                LogLevel::Error,
                                format!(
                                    "Failed to encode login play packet for {}: {}",
                                    player.username, e
                                )
                                .as_str(),
                            );
                            socket.shutdown().await?;
                            break;
                        }
                    };

                    socket.write_all(&encoded).await?;

                    crate::log::log(
                        LogLevel::Debug,
                        format!("Switching {} to play state", player.username).as_str(),
                    );

                    play::play(socket, player).await?;
                    break;
                }

                _ => {}
            },

            Ok(Ok(None)) => {}

            Err(e) => {
                crate::log::log(
                    LogLevel::Error,
                    format!(
                        "{} has timed out during configuration state: {}",
                        player.username, e
                    )
                    .as_str(),
                );

                socket.shutdown().await?;
                break;
            }

            Ok(Err(e)) => {
                crate::log::log(
                    LogLevel::Error,
                    format!(
                        "Error while reading packet from {} during configuration state: {}",
                        player.username, e
                    )
                    .as_str(),
                );

                socket.shutdown().await?;
                break;
            }
        }
    }

    Ok(())
}
