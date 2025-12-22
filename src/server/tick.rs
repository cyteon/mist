use tokio::time;
use fancy_log::{LogLevel, log};
use tokio::time::Duration;

use crate::server::save::save;

pub async fn start_tick_loop() -> anyhow::Result<()> {
    log(LogLevel::Info, "Tick loop started");

    let mut interval = time::interval(Duration::from_millis(50)); // 20 tps
    let mut ticks_until_autosave = 0; // so it autosaves on start

    loop {
        if ticks_until_autosave == 0 {
            ticks_until_autosave = 6000;
            save();
        } else {
            ticks_until_autosave -= 1;
        }

        interval.tick().await;
    }
}