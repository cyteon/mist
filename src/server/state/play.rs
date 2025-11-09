use std::time::Duration;

use fancy_log::{LogLevel, log};
use tokio::{io::AsyncWriteExt, net::TcpStream, time::timeout};

use crate::{
    net::packet::{
        ProtocolState, 
        read_packet
    }, 
    
    server::{
        encryption::EncryptedStream, 
        state::login::Player
    }
};

pub async fn play(mut socket: EncryptedStream<TcpStream>, player: Player) -> anyhow::Result<()> {
    log(LogLevel::Debug, format!("{} has entered the play state", player.name).as_str());

    loop {
        match timeout(Duration::from_secs(15), read_packet(&mut socket, &ProtocolState::Play)).await {
            Ok(Ok(Some(packet))) => {
                match packet {
                    

                    _ => { }
                }
            },

            Ok(Ok(None)) => { },

            Err(e) => { 
                log(
                    LogLevel::Error, 
                    format!("{} has timed out during play state: {}", player.name, e).as_str()
                );

                socket.shutdown().await?; 
                break; 
            }
                
            Ok(Err(e)) => {
                log(
                    LogLevel::Error, 
                    format!("Error while reading packet from {} during play state: {}", player.name, e).as_str()
                );

                socket.shutdown().await?; 
                break; 
            }
        }
    }

    Ok(())
}