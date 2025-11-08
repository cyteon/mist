use tokio::net::TcpStream;

use crate::{net::packets::serverbound::handshake::{HandshakePacket, read_handshake}, server::state};

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