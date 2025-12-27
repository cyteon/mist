use tokio::net::TcpListener;
use fancy_log::LogLevel;
use tokio::time::{timeout, Duration};

use crate::server::conn::handle_conn;

pub async fn start_listener() -> anyhow::Result<()> {
    let addr = format!("{}:{}", 
        crate::config::SERVER_CONFIG.host,
        crate::config::SERVER_CONFIG.port
    );

    let listener = match timeout(Duration::from_secs(1), TcpListener::bind(&addr)).await {
        Ok(Ok(listener)) => listener,

        Ok(Err(e)) => {
            crate::log::log(LogLevel::Error, format!("Failed to bind to {}: {}", addr, e).as_str());
            panic!("Failed to bind to address: {}", e);
        }

        Err(_) => {
            crate::log::log(LogLevel::Error, format!("Timeout while binding to {}", addr).as_str());
            panic!("Bind timeout");
        }
    };
    
    crate::log::log(LogLevel::Info, format!("Listening on {}", &addr).as_str());

    loop {
        let (socket, addr) = listener.accept().await?;
        socket.set_nodelay(true)?;

        crate::log::log(LogLevel::Debug, format!("New connection from {}", addr).as_str());

        tokio::spawn(async move {
            if let Err(e) = handle_conn(socket) .await {
                crate::log::log(LogLevel::Error, format!("Error handling connection from {}: {}", addr, e).as_str());
            }
        });
    }
}