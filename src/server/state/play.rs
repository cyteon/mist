use std::time::Duration;

use fancy_log::{LogLevel, log};
use tokio::{io::AsyncWriteExt, net::TcpStream, time::{self, timeout}};

use crate::{
    net::{packet::{
        ProtocolState, 
        read_packet
    }, packets::clientbound::{keep_alive::send_keep_alive, sync_player_position::send_sync_player_position}}, 
    
    server::{conn::PLAYER_SOCKET_MAP, encryption::EncryptedStream},
    types::player::Player
};

use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn play(socket: EncryptedStream<TcpStream>, player: Player) -> anyhow::Result<()> {
    log(LogLevel::Debug, format!("{} has entered the play state", player.name).as_str());

    let socket = Arc::new(Mutex::new(socket));

    let keep_alive_future ={
        let socket = Arc::clone(&socket);

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(15));

            loop {
                interval.tick().await;

                let mut socket = socket.lock().await;
                send_keep_alive(&mut *socket).await.unwrap();
            }
        })
    };

    send_sync_player_position(&mut *socket.lock().await, &player).await?;

    loop {
        let mut socket_guard = socket.lock().await;

        match timeout(Duration::from_secs(20), read_packet(&mut *socket_guard, &ProtocolState::Play)).await {
            Ok(Ok(Some(packet))) => {
                match packet {
                    _ => { }
                }
                socket_guard.shutdown().await?; 
                break; 
            },

            Ok(Ok(None)) => { },

            Err(e) => { 
                log(
                    LogLevel::Error, 
                    format!("{} has timed out during play state: {}", player.name, e).as_str()
                );

                socket_guard.shutdown().await?;
                keep_alive_future.abort();
                break; 
            }
                
            Ok(Err(e)) => {
                log(
                    LogLevel::Error, 
                    format!("Error while reading packet from {} during play state: {}", player.name, e).as_str()
                );

                socket_guard.shutdown().await?;
                keep_alive_future.abort();
                break; 
            }
        }
    }

    Ok(())
}