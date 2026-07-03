use std::pin::Pin;

use crate::types::colors::{GREEN, RED, YELLOW};
use crate::types::player::Player;

pub fn run<'a>(
    _args: &'a [&'a str],
    player: &'a mut Player,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let tps_5s = crate::server::tick::TPS_5S.load(std::sync::atomic::Ordering::Relaxed);
        let tps_1m = crate::server::tick::TPS_1M.load(std::sync::atomic::Ordering::Relaxed);
        let tps_5m = crate::server::tick::TPS_5M.load(std::sync::atomic::Ordering::Relaxed);

        let tps_5s_color = if tps_5s >= 18 {
            GREEN
        } else if tps_5s >= 15 {
            YELLOW
        } else {
            RED
        };
        let tps_1m_color = if tps_1m >= 18 {
            GREEN
        } else if tps_1m >= 15 {
            YELLOW
        } else {
            RED
        };
        let tps_5m_color = if tps_5m >= 18 {
            GREEN
        } else if tps_5m >= 15 {
            YELLOW
        } else {
            RED
        };

        player
            .send_system_message(format!(
                "TPS (last 5s): {}{}§f, TPS (last 1m): {}{}§f, TPS (last 5m): {}{}§f",
                tps_5s_color, tps_5s, tps_1m_color, tps_1m, tps_5m_color, tps_5m
            ))
            .await
    })
}
