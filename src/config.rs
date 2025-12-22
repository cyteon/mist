use std::process::exit;
use once_cell::sync::Lazy;
use serde::{Deserialize};
use fancy_log::{LogLevel, log};

pub static SERVER_CONFIG: Lazy<ServerConfig> = Lazy::new(|| {
    load_config()
});

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,

    pub motd: String,
    pub max_players: u32,
    pub online_mode: bool,

    pub world_name: String,
}

pub fn load_config() -> ServerConfig {
    log(LogLevel::Info, "Loading config");

    let path = "config.toml";
    let file_exists = std::path::Path::new(path).exists();

    if file_exists {
        let config_str = std::fs::read_to_string(path);

        if config_str.is_err() {
            log(LogLevel::Error, "Failed to read config");
            log(LogLevel::Error, "Stopping server");

            exit(1);
        }

        match toml::from_str::<ServerConfig>(&config_str.unwrap()) {
            Ok(config) => config,
            Err(e) => {
                log(LogLevel::Error, format!("Failed to parse config:\n\n{}", e).as_str());
                log(LogLevel::Info, "Stopping server");

                exit(1);
            }
        }
    } else {
        let default_config = r#"# the host the server will bind to
host = "0.0.0.0"

# the port the server will listen on
port = 25565

# server details
motd = "An mist server"
max_players = 10
online_mode = true

world_name = "world""#;
        
        if std::fs::write(path, default_config).is_err() {
            log(LogLevel::Error, "Failed to write default config");
            log(LogLevel::Error, "Stopping server");

            exit(1);
        } else {
            log(LogLevel::Info, "Default config created");
        }

        match toml::from_str::<ServerConfig>(default_config) {
            Ok(config) => config,
            Err(_) => {
                log(LogLevel::Error, "Failed to parse default config");
                log(LogLevel::Error, "Stopping server");

                exit(1);
            }
        }
    }
}