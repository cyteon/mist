use fancy_log::{LogLevel, set_log_level};
use once_cell::sync::Lazy;

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

pub static SERVER_PROTOCOL_VERSION: i32 = 773;
pub static SERVER_VERSION: &str = "1.21.10";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_log_level(LogLevel::Debug);
    log::log(LogLevel::Info, format!("Starting mist for minecraft {}/{}", SERVER_VERSION, SERVER_PROTOCOL_VERSION).as_str());

    // just to ensure that config has loaded
    log::log(LogLevel::Info, format!("Server motd is \"{}\"", &config::SERVER_CONFIG.motd).as_str());

    tokio::spawn(async {
        tokio::signal::ctrl_c().await.ok();
        log::log(LogLevel::Info, "Received shutdown signal, stopping server...");
        
        crate::server::save::save().await;

        std::process::exit(0);
    });

    server::run::run().await
}
