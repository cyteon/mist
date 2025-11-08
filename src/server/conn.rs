use std::time::Duration;

use fancy_log::{LogLevel, log};
use tokio::{io::AsyncWriteExt, net::TcpStream, time::timeout};

use crate::net::{
    packet::{ClientPacket, ProtocolState, read_packet}, 
    packets::{
        clientbound::{disconnect::send_disconnect_login, encryption_request::send_encryption_request, pong::send_pong, status_response::send_status_response}, 
        serverbound::{handshake::{HandshakePacket, read_handshake}, login_start::read_login_start}
    }
};


pub async fn handle_conn(mut socket: TcpStream) -> anyhow::Result<()> {
    let mut state = ProtocolState::Handshake;

    let handshake: HandshakePacket = read_handshake(&mut socket).await?;

    match handshake.next_state {
        1 => {
            state = ProtocolState::Status;
            send_status_response(&mut socket).await?;
        },
        
        2 => {
            state = ProtocolState::Login;

            if handshake.protocol_version != 773 {
                send_disconnect_login(
                    &mut socket, 
                    "Unsupported version. Please use Minecraft 1.21.10"
                ).await?;
            }
        },

        _ => {
            anyhow::bail!("Invalid next state: {}", handshake.next_state);
        }
    }

    loop {
        match timeout(Duration::from_secs(2), read_packet(&mut socket, &state)).await {
            Ok(Ok(Some(packet))) => {
                match packet {
                    ClientPacket::Ping => {
                        send_pong(&mut socket).await?;
                        log(LogLevel::Debug, "Responded to ping request");
                    },

                    ClientPacket::LoginStart => {
                        let login_start = read_login_start(&mut socket).await?;
                        log(LogLevel::Info, format!("{} ({}) is connecting", login_start.name, login_start.uuid).as_str());

                        send_encryption_request(&mut socket).await?;
                    }

                    _ => { }
                }
            },

            Ok(Ok(None)) => { }

            // timeout err
            Err(e) => {
                socket.shutdown().await.ok();
                log(LogLevel::Warn, format!("Connection timed out: {}", e).as_str());
                break;
            }

            // read err
            Ok(Err(e)) => {
                socket.shutdown().await.ok();
                log(LogLevel::Error, format!("Error reading packet: {}", e).as_str());
                break;
            }
        }   
    }

    Ok(())
}