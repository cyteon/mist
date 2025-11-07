use std::time::Duration;

use fancy_log::{LogLevel, log};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, time::timeout};

use crate::net::{codec::write_var, packet::{ClientPacket, HandshakePacket, ProtocolState, read_handshake, read_packet, send_status_response}};


pub async fn handle_conn(mut socket: TcpStream) -> anyhow::Result<()> {
    let mut state = ProtocolState::Handshake;

    let handshake: HandshakePacket = read_handshake(&mut socket).await?;

    match handshake.next_state {
        1 => {
            state = ProtocolState::Status;
            send_status_response(&mut socket).await?;
            socket.flush().await?;
        },
        
        2 => {
            state = ProtocolState::Login;
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
                        let mut packet_data = vec![];
                        write_var(&mut packet_data, 0x01).await?;
                        packet_data.extend_from_slice([0u8; 8].as_ref());

                        write_var(&mut socket, packet_data.len() as i32).await?;
                        socket.write_all(&packet_data).await?;
                        socket.flush().await?;

                        log(LogLevel::Debug, "Responded to ping request");
                    },

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