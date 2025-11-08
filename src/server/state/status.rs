use std::time::Duration;

use fancy_log::{LogLevel, log};
use tokio::{
    io::AsyncWriteExt, 
    net::TcpStream, 
    time::timeout
};

use crate::net::{
    packet::{
        ClientPacket, 
        ProtocolState, 
        read_packet
    }, 
    
    packets::clientbound::{
        pong::send_pong, 
        status_response::send_status_response
    }
};

pub async fn status(mut socket: TcpStream) -> anyhow::Result<()> {
    send_status_response(&mut socket).await?;

    loop {
        match timeout(Duration::from_secs(2), read_packet(&mut socket, &ProtocolState::Status)).await {
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
}