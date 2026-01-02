use tokio::time;
use fancy_log::LogLevel;
use tokio::time::Duration;

use crate::server::save::save;

pub async fn start_tick_loop() -> anyhow::Result<()> {
    crate::log::log(LogLevel::Info, "Tick loop started");

    let mut interval = time::interval(Duration::from_millis(50)); // 20 tps
    let mut ticks_until_autosave = 100; // so it autosaves 5 seconds after start

    let mut last_tps_check = std::time::Instant::now();
    let mut ticks = 0;

    loop {
        if ticks_until_autosave == 0 {
            ticks_until_autosave = 6000; // 5 mins
            save().await;
        } else {
            ticks_until_autosave -= 1;
        }

        ticks += 1;
        if last_tps_check.elapsed().as_secs() >= 5 {
            let elapsed = last_tps_check.elapsed().as_secs_f64();
            let tps = ticks as f64 / elapsed;
            crate::log::log(LogLevel::Debug , &format!("TPS (last 5s): {:.2}", tps));
            last_tps_check = std::time::Instant::now();
            ticks = 0;
        }

        for player in crate::server::state::play::PLAYERS.read().await.values() {
            let mut player_lock = player.lock().await;
            player_lock.tick().await;

            dbg!(player_lock.x, player_lock.y, player_lock.z);
        }

        interval.tick().await;
    }
}