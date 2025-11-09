use fancy_log::{LogLevel, log};
use tokio::net::TcpStream;

use crate::server::{encryption::EncryptedStream, state::login::Player};

pub async fn play(mut socket: EncryptedStream<TcpStream>, player: Player) -> anyhow::Result<()> {
    log(LogLevel::Debug, format!("{} has entered the play state", player.name).as_str());

    loop {
        
    }

    Ok(())
}