use tokio::time;
use fancy_log::{LogLevel, log};
use tokio::time::Duration;

pub async fn start_tick_loop() -> anyhow::Result<()> {
    log(LogLevel::Info, "Tick loop started");

    let mut interval = time::interval(Duration::from_millis(50));

    loop {
        interval.tick().await;
    }
}