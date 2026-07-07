use std::time::Duration;

use log::{LogLevel, set_log_level};
use once_cell::sync::Lazy;
use rustyline::{DefaultEditor, error::ReadlineError};
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::server::commands::{CommandInvoker, handle_command};

mod config;
mod log;

mod net;
mod server;
mod types;
mod world;

pub static RSA_PRIVATE_KEY: Lazy<rsa::RsaPrivateKey> = Lazy::new(|| {
    rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 1024).expect("Failed to generate RSA key")
});

pub static RSA_PUBLIC_KEY: Lazy<rsa::RsaPublicKey> =
    Lazy::new(|| rsa::RsaPublicKey::from(&*RSA_PRIVATE_KEY));

pub static SERVER_PROTOCOL_VERSION: i32 = 774;
pub static SERVER_VERSION: &str = "1.21.11";

enum ConsoleEvent {
    Line(String),
    Shutdown,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "console")]
    console_subscriber::init();

    set_log_level(LogLevel::Debug);
    log::log(
        LogLevel::Info,
        format!(
            "Starting mist for minecraft {}/{}",
            SERVER_VERSION, SERVER_PROTOCOL_VERSION
        )
        .as_str(),
    );

    // just to ensure that config has loaded
    log::log(
        LogLevel::Info,
        format!("Server motd is \"{}\"", &config::SERVER_CONFIG.motd).as_str(),
    );

    let mut rl = DefaultEditor::new()?;
    let printer = rl.create_external_printer()?;

    log::set_printer(printer);

    let (console_tx, mut console_rx) = mpsc::unbounded_channel::<ConsoleEvent>();

    tokio::task::spawn_blocking(move || {
        loop {
            match rl.readline("> ") {
                Ok(line) => {
                    let _ = rl.add_history_entry(&line);

                    if console_tx.send(ConsoleEvent::Line(line)).is_err() {
                        break;
                    }
                }

                Err(ReadlineError::Interrupted) => {
                    let _ = console_tx.send(ConsoleEvent::Shutdown);
                    break;
                }

                Err(ReadlineError::Eof) => break,
                Err(_) => {}
            }
        }
    });

    tokio::spawn(async move {
        while let Some(event) = console_rx.recv().await {
            match event {
                ConsoleEvent::Line(line) => {
                    let mut invoker = CommandInvoker::Console;
                    let _ = handle_command(line, &mut invoker).await;
                }

                ConsoleEvent::Shutdown => {
                    log::log(
                        LogLevel::Info,
                        "Received shutdown signal, stopping server...\n",
                    );

                    if timeout(Duration::from_secs(5), crate::server::run::stop())
                        .await
                        .is_err()
                    {
                        log::log(LogLevel::Error, "Timeout while stopping server :(\n");
                        log::log(LogLevel::Error, "Killing...\n");
                    }

                    std::process::exit(0);
                }
            }
        }
    });

    server::run::run().await
}
