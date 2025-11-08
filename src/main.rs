use fancy_log::{LogLevel, log, set_log_level};
use once_cell::sync::Lazy;

mod config;
mod server;
mod net;

pub static RSA_PRIVATE_KEY: Lazy<rsa::RsaPrivateKey> = Lazy::new(|| {
    rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 1024).expect("Failed to generate RSA key")
});

pub static RSA_PUBLIC_KEY: Lazy<rsa::RsaPublicKey> = Lazy::new(|| {
    rsa::RsaPublicKey::from(&*RSA_PRIVATE_KEY)
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_log_level(LogLevel::Debug);

    // just to ensure that config has loaded
    log(LogLevel::Info, format!("Server motd is \"{}\"", &config::SERVER_CONFIG.motd).as_str());

    server::run::run().await
}
