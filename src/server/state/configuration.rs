use std::time::Duration;

use fancy_log::LogLevel;
use tokio::{io::AsyncWriteExt, net::TcpStream, time::timeout};

use crate::{
    net::{
        packet::{
            ClientPacket, ProtocolState, read_packet
        }, 
        
        packets::{
            clientbound::{
                finish_configuration::send_finish_configuration, known_packs::send_known_packs, login_play::send_login_play, plugin_message::send_plugin_message, registry_data::send_all_registers
            }, 
            
            serverbound::known_packs::read_known_packs
        }
    }, 
    
    server::{
        encryption::EncryptedStream, state::play
    }, types::player::Player
};

pub async fn configuration(mut socket: EncryptedStream<TcpStream>, player: Player) -> anyhow::Result<()> {
    crate::log::log(LogLevel::Debug, format!("{} has entered the configuration state", player.name).as_str());

    send_plugin_message(&mut socket).await?;
    send_known_packs(&mut socket).await?;

    crate::log::log(LogLevel::Debug, format!("Sent known packs to {}", player.name).as_str());
   
    loop {
        match timeout(Duration::from_secs(15), read_packet(&mut socket, &ProtocolState::Configuration)).await {
            Ok(Ok(Some(packet))) => {
                match packet {
                    ClientPacket::KnownPacks(mut cursor) => {
                        read_known_packs(&mut cursor).await?;
                        crate::log::log(LogLevel::Debug, format!("{} has sent known packs", player.name).as_str());

                        send_all_registers(&mut socket).await?;
                        crate::log::log(LogLevel::Debug, format!("Sent registry data to {}", player.name).as_str());

                        send_finish_configuration(&mut socket).await?;
                        crate::log::log(LogLevel::Debug, format!("Sent finish configuration to {}", player.name).as_str());
                    },

                    ClientPacket::AcknowledgeFinishConfiguration => {
                        crate::log::log(LogLevel::Debug, format!("{} has finished configuration", player.name).as_str());
                        
                        send_login_play(&mut socket).await?;
                        crate::log::log(LogLevel::Debug, format!("Switching {} to play state", player.name).as_str());

                        play::play(socket, player).await?;
                        break;
                    },

                    _ => { }
                }
            },

            Ok(Ok(None)) => { },

            Err(e) => {
                crate::log::log(
                    LogLevel::Error, 
                    format!("{} has timed out during configuration state: {}", player.name, e).as_str()
                );
                
                socket.shutdown().await?; 
                break; 
            }

            Ok(Err(e)) => {
                crate::log::log(
                    LogLevel::Error, 
                    format!("Error while reading packet from {} during configuration state: {}", player.name, e).as_str()
                );

                socket.shutdown().await?; 
                break; 
            }
        }
    }

    Ok(())
}