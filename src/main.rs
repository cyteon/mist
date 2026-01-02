use std::time::Duration;

use fancy_log::{LogLevel, set_log_level};
use once_cell::sync::Lazy;
use tokio::time::timeout;

mod config;
mod log;

mod server;
mod net;
mod types;
mod world;

pub static RSA_PRIVATE_KEY: Lazy<rsa::RsaPrivateKey> = Lazy::new(|| {
    rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 1024).expect("Failed to generate RSA key")
});

pub static RSA_PUBLIC_KEY: Lazy<rsa::RsaPublicKey> = Lazy::new(|| {
    rsa::RsaPublicKey::from(&*RSA_PRIVATE_KEY)
});

pub static SERVER_PROTOCOL_VERSION: i32 = 774;
pub static SERVER_VERSION: &str = "1.21.11";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    #[cfg(feature = "console")]
    console_subscriber::init();

    set_log_level(LogLevel::Info);
    log::log(LogLevel::Info, format!("Starting mist for minecraft {}/{}", SERVER_VERSION, SERVER_PROTOCOL_VERSION).as_str());

    // just to ensure that config has loaded
    log::log(LogLevel::Info, format!("Server motd is \"{}\"", &config::SERVER_CONFIG.motd).as_str());

    tokio::spawn(async {
        tokio::signal::ctrl_c().await.ok();
        log::log(LogLevel::Info, "Received shutdown signal, stopping server...");
        
        match timeout(Duration::from_secs(5), crate::server::save::save()).await {
            Err(_) => log::log(LogLevel::Error, "Timeout while saving server :("),

            _ => {}
        }

        std::process::exit(0);
    });

    server::run::run().await
}
