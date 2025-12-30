use fancy_log::LogLevel;
use tokio::net::TcpStream;

use crate::{
    config::SERVER_CONFIG, 

    net::packets::{
        clientbound::{
            disconnect::send_disconnect_login, 
            encryption_request::send_encryption_request, 
            login_success::send_login_success
        }, 
        
        serverbound::{
            encryption_response::read_encryption_response, 
            handshake::HandshakePacket, 
            login_acknowledged::read_login_acknowledged, 
            login_start::read_login_start
        }
    }, 
    
    server::{
        auth::authenticate_player, 
        encryption::EncryptedStream, state::configuration
    }, 
    
    types::player::Player
};

pub async fn login(mut socket: TcpStream, handshake: HandshakePacket) -> anyhow::Result<()> {
    let mut player: Option<Player>;

    if handshake.protocol_version != crate::SERVER_PROTOCOL_VERSION {
        send_disconnect_login(
            &mut socket, 
            format!(
                "Incompatible minecraft version. Server is running {} (protocol {})", 
                crate::SERVER_VERSION,
                crate::SERVER_PROTOCOL_VERSION
            ).as_str()
        ).await?;
    }

    let login_start = read_login_start(&mut socket).await?;
    crate::log::log(LogLevel::Info, format!("{} ({}) is connecting", login_start.name, login_start.uuid).as_str());

    player = Some(Player::new(
        login_start.uuid.clone(),
        login_start.name.clone()
    ));

    send_encryption_request(&mut socket).await?;

    let encryption_response = read_encryption_response(&mut socket).await?;

    let mut socket = EncryptedStream::new(
        socket, 
        encryption_response.shared_secret.clone().as_slice(), 
    );

    player.as_mut().unwrap().shared_secret = Some(encryption_response.shared_secret.clone());

    if SERVER_CONFIG.online_mode {
        let player_name = player.as_ref().unwrap().username.clone();
        let player_data = authenticate_player(&player_name, encryption_response.shared_secret.clone()).await?;

        player.as_mut().unwrap().textures = Some(player_data.textures);
        player.as_mut().unwrap().texture_signature = Some(player_data.texture_signature);
    }

    send_login_success(&mut socket, &player.clone().unwrap()).await?;
    crate::log::log(LogLevel::Debug, format!("Sent login success to {}", player.as_ref().unwrap().username).as_str());
    
    read_login_acknowledged(&mut socket).await?;
    crate::log::log(LogLevel::Debug, format!("{} sent login acknowledged", player.as_ref().unwrap().username).as_str());

    configuration::configuration(socket, player.unwrap()).await?;

    Ok(())
}