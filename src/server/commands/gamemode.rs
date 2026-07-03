use std::pin::Pin;

use crate::types::colors::{GREEN, RED};
use crate::types::player::Player;

pub fn run<'a>(
    args: &'a [&'a str],
    player: &'a mut Player,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        if args.len() < 1 {
            player
                .send_system_message(format!("{}Usage: /gamemode <mode>", RED))
                .await?;
            return Ok(());
        }

        let mode_str = args[0].to_lowercase();
        let gamemode = match mode_str.as_str() {
            "survival" | "s" | "0" => crate::types::player::Gamemode::Survival,
            "creative" | "c" | "1" => crate::types::player::Gamemode::Creative,
            "adventure" | "a" | "2" => crate::types::player::Gamemode::Adventure,
            "spectator" | "sp" | "3" => crate::types::player::Gamemode::Spectator,
            _ => {
                player
                    .send_system_message(format!("{}Unknown gamemode: {}", RED, mode_str))
                    .await?;
                return Ok(());
            }
        };

        player.set_gamemode(gamemode).await?;
        player
            .send_system_message(format!(
                "{}Your gamemode has been set to {}",
                GREEN, mode_str
            ))
            .await
    })
}
