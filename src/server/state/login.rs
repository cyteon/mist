use fancy_log::LogLevel;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::{
    config::SERVER_CONFIG,
    net::{
        packet::encode_packet,
        packets::{
            clientbound::{
                disconnect::send_disconnect_login, encryption_request::send_encryption_request,
                login_success::send_login_success, set_compression::send_set_compression,
            },
            serverbound::{
                encryption_response::read_encryption_response, handshake::HandshakePacket,
                login_acknowledged::read_login_acknowledged, login_start::read_login_start,
            },
        },
    },
    server::{auth::authenticate_player, encryption::EncryptedStream, state::configuration},
    types::player::Player,
};

pub async fn login(mut socket: TcpStream, handshake: HandshakePacket) -> anyhow::Result<()> {
    if handshake.protocol_version != crate::SERVER_PROTOCOL_VERSION {
        send_disconnect_login(
            &mut socket,
            format!(
                "Incompatible minecraft version. Server is running {} (protocol {})\nYou connected with protocol version {}",
                crate::SERVER_VERSION,
                crate::SERVER_PROTOCOL_VERSION,
                handshake.protocol_version
            ).as_str()
        ).await?;
    }

    let current_players = crate::server::state::play::PLAYERS.read().await.len();
    if current_players >= SERVER_CONFIG.max_players as usize {
        send_disconnect_login(&mut socket, "The server is full! Please try again later.").await?;

        return Ok(());
    }

    let login_start = read_login_start(&mut socket).await?;

    let mut player = Player::new(login_start.uuid.clone(), login_start.username.clone()).await;

    send_encryption_request(&mut socket).await?;

    let encryption_response = read_encryption_response(&mut socket).await?;

    let mut socket = EncryptedStream::new(socket, encryption_response.clone().as_slice());

    player.shared_secret = Some(encryption_response.clone());

    if SERVER_CONFIG.online_mode {
        let player_name = player.username.clone();
        let player_data = authenticate_player(&player_name, encryption_response.clone()).await?;

        player.username = player_data.username; // we alr know username, but use mojang as an source of truth
        player.textures = Some(player_data.textures);
        player.texture_signature = Some(player_data.texture_signature);
    }

    send_set_compression(&mut socket).await?;
    crate::log::log(
        LogLevel::Debug,
        format!("Sent set compression to {}", player.username).as_str(),
    );

    let mut buffer = Vec::new();
    send_login_success(&mut buffer, &player).await?;

    let encoded = match encode_packet(&buffer) {
        Ok(packet) => packet,

        Err(e) => {
            crate::log::log(
                LogLevel::Error,
                format!(
                    "Failed to encode login success packet for {}: {}",
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
        format!("Sent login success to {}", player.username).as_str(),
    );

    read_login_acknowledged(&mut socket).await?;

    crate::log::log(
        LogLevel::Debug,
        format!("{} sent login acknowledged", player.username).as_str(),
    );

    configuration::configuration(socket, player).await?;

    Ok(())
}
