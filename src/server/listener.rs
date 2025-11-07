use tokio::net::TcpListener;
use fancy_log::{LogLevel, log};

pub async fn start_listener() -> anyhow::Result<()> {
    let addr = format!("{}:{}", 
        crate::config::SERVER_CONFIG.host,
        crate::config::SERVER_CONFIG.port
    );

    let listener = TcpListener::bind(&addr).await?;
    log(LogLevel::Info, format!("Listening on {}", &addr).as_str());

    loop {
        let (socket, addr) = listener.accept().await?;
        log(LogLevel::Info, format!("New connection from {}", addr).as_str());

        tokio::spawn(async move {
            
        });
    }
}