use std::{iter::Map, time::Duration};

use fancy_log::{LogLevel, log};
use once_cell::sync::Lazy;
use tokio::{io::AsyncWriteExt, net::TcpStream, time::timeout};

use crate::{config::SERVER_CONFIG, net::{
    packet::{ClientPacket, ProtocolState, read_packet}, 
    packets::{
        clientbound::{disconnect::send_disconnect_login, encryption_request::send_encryption_request, login_success::send_login_success, pong::send_pong, status_response::send_status_response}, 
        serverbound::{encryption_response::read_encryption_response, handshake::{HandshakePacket, read_handshake}, login_start::read_login_start}
    }
}, server::{auth::authenticate_player, encryption::EncryptedStream}};

#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub uuid: String,
    pub shared_secret: Option<Vec<u8>>,
    pub skin_texture: Option<String>,
}

pub async fn handle_conn(mut socket: TcpStream) -> anyhow::Result<()> {
    let mut state = ProtocolState::Handshake;

    let handshake: HandshakePacket = read_handshake(&mut socket).await?;

    let mut player: Option<Player> = None;

    match handshake.next_state {
        1 => {
            state = ProtocolState::Status;
            send_status_response(&mut socket).await?;

            loop {
                match timeout(Duration::from_secs(2), read_packet(&mut socket, &state)).await {
                    Ok(Ok(Some(packet))) => {
                        match packet {
                            ClientPacket::Ping => {
                                send_pong(&mut socket).await?;
                                log(LogLevel::Debug, "Responded to ping request");
                            },

                            _ => { }
                        }
                    },

                    Ok(Ok(None)) => { },
                    Err(_) => { socket.shutdown().await.ok(); }
                    Ok(Err(_)) => { socket.shutdown().await.ok(); }
                }
            }
        },
        
        2 => {
            state = ProtocolState::Login;

            if handshake.protocol_version != 773 {
                send_disconnect_login(
                    &mut socket, 
                    "Unsupported version. Please use Minecraft 1.21.10"
                ).await?;
            }

            let login_start = read_login_start(&mut socket).await?;
            log(LogLevel::Info, format!("{} ({}) is connecting", login_start.name, login_start.uuid).as_str());

            player = Some(Player {
                name: login_start.name,
                uuid: login_start.uuid,
                shared_secret: None,
                skin_texture: None,
            });

            send_encryption_request(&mut socket).await?;

            let encryption_response = read_encryption_response(&mut socket).await?;

            let mut socket = EncryptedStream::new(
                socket, 
                encryption_response.shared_secret.clone().as_slice(), 
            );

            player.as_mut().unwrap().shared_secret = Some(encryption_response.shared_secret.clone());

            if SERVER_CONFIG.online_mode {
                let player_name = player.as_ref().unwrap().name.clone();
                let player_data = authenticate_player(&player_name, encryption_response.shared_secret.clone()).await?;

                player.as_mut().unwrap().skin_texture = Some(player_data.skin_texture);
            }

            send_login_success(&mut socket, player.clone().unwrap().name.as_str(), player.clone().unwrap().uuid.as_str()).await?;
        },

        _ => {
            anyhow::bail!("Invalid next state: {}", handshake.next_state);
        }
    }

    Ok(())
}