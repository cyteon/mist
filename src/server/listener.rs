use tokio::net::TcpListener;
use fancy_log::{LogLevel, log};

use crate::server::conn::handle_conn;

pub async fn start_listener() -> anyhow::Result<()> {
    let addr = format!("{}:{}", 
        crate::config::SERVER_CONFIG.host,
        crate::config::SERVER_CONFIG.port
    );

    let listener = TcpListener::bind(&addr).await?;
    log(LogLevel::Info, format!("Listening on {}", &addr).as_str());

    loop {
        let (socket, addr) = listener.accept().await?;
        socket.set_nodelay(true)?;

        log(LogLevel::Debug, format!("New connection from {}", addr).as_str());

        tokio::spawn(async move {
            if let Err(e) = handle_conn(socket) .await {
                log(LogLevel::Error, format!("Error handling connection from {}: {}", addr, e).as_str());
            }
        });
    }
}