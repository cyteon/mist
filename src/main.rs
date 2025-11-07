use fancy_log::{LogLevel, log, set_log_level};

mod config;
mod server;
mod net;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_log_level(LogLevel::Debug);

    // just to ensure that config has loaded
    log(LogLevel::Info, format!("Server motd is \"{}\"", &config::SERVER_CONFIG.motd).as_str());

    server::run::run().await
}
