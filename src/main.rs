use fancy_log::{log, LogLevel};

mod config;
mod net;

#[tokio::main]
async fn main() {
    // just to ensure that config has loaded
    log(LogLevel::Info, format!("Server name is \"{}\"", &config::SERVER_CONFIG.server_name).as_str());


}
