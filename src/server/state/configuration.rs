use std::time::Duration;

use fancy_log::{LogLevel, log};
use tokio::{io::AsyncWriteExt, net::TcpStream, time::timeout};

use crate::{
    net::{codec::write_var, packet::{ClientPacket, ProtocolState, read_packet}, packets::clientbound::{finish_configuration::send_finish_configuration, known_packs::send_known_packs, regristry_data::send_regristry_data}}, 
    
    server::{
        encryption::EncryptedStream, state::{login::Player, play}
    }
};

pub async fn configuration(mut socket: EncryptedStream<TcpStream>, player: Player) -> anyhow::Result<()> {
    log(LogLevel::Debug, format!("{} has entered the configuration state", player.name).as_str());

    send_known_packs(&mut socket).await?;
   
    loop {
        match timeout(Duration::from_secs(15), read_packet(&mut socket, &ProtocolState::Configuration)).await {
            Ok(Ok(Some(packet))) => {
                match packet {
                    ClientPacket::KnownPacks => {
                        send_regristry_data(&mut socket).await?;

                        // config finished as there is A LOT we dont have
                        send_finish_configuration(&mut socket).await?;
                        log(LogLevel::Debug, format!("Sent finish configuration to {}", player.name).as_str());
                    },

                    ClientPacket::AcknowledgeFinishConfiguration => {
                        log(LogLevel::Debug, format!("{} has finished configuration", player.name).as_str());
                        play::play(socket, player).await?;
                        break;
                    },

                    _ => { }
                }
            },

            Ok(Ok(None)) => { },
            Err(_) => { socket.shutdown().await?; }
            Ok(Err(_)) => { socket.shutdown().await?; }
        }
    }

    Ok(())
}