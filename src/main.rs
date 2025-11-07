use fancy_log::{log, LogLevel};

mod config;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // just to ensure that config has loaded
    log(LogLevel::Info, format!("Server name is \"{}\"", &config::SERVER_CONFIG.server_name).as_str());

    server::run::run().await
}
