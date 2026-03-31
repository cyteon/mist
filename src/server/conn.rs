use std::{collections::HashMap, sync::Arc};
use once_cell::sync::Lazy;
use tokio::{net::{TcpStream, tcp::OwnedWriteHalf}, sync::{Mutex, RwLock, mpsc}};

use crate::{
    net::packets::serverbound::handshake::{
        HandshakePacket, 
        read_handshake
    }, 

    server::{
        encryption::EncryptedWriter, 
        state
    }
};

pub static PLAYER_SOCKET_MAP: Lazy<RwLock<HashMap<String, mpsc::UnboundedSender<Vec<u8>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn handle_conn(mut socket: TcpStream) -> anyhow::Result<()> {
    let handshake: HandshakePacket = read_handshake(&mut socket).await?;

    match handshake.next_state {
        1 => { state::status::status(socket).await?; },
        
        2 => { state::login::login(socket, handshake).await?; },

        _ => {
            anyhow::bail!("Invalid next state: {}", handshake.next_state);
        }
    }

    Ok(())
}