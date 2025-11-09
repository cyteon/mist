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
            Err(_) => { 
                log(LogLevel::Info, format!("{} has timed out during play state", player.name).as_str());
                socket.shutdown().await?; 
                break; 
            }
                
            Ok(Err(_)) => {
                log(LogLevel::Info, format!("{} has encountered an error during play state", player.name).as_str());
                socket.shutdown().await?; 
                break; 
            }
        }
    }

    Ok(())
}