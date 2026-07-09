use std::pin::Pin;

use crate::server::commands::CommandInvoker;
use crate::server::state::play::{NAME_TO_UUID, PLAYERS};
use crate::types::colors::{GREEN, RED};

pub fn run<'a, 'b>(
    args: &'a [&'a str],
    invoker: &'a mut CommandInvoker<'b>,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        if args.len() < 2 {
            invoker
                .send_message(format!("{}Usage: /gamemode <player> <gamemode>", RED))
                .await?;

            return Ok(());
        }

        let uuid = {
            let map = NAME_TO_UUID.read().await;
            map.get(&args[0].to_lowercase()).cloned()
        };

        let Some(uuid) = uuid else {
            invoker
                .send_message(format!("{}Player not found: {}", RED, args[0]))
                .await?;
            return Ok(());
        };

        let mode_str = args[1].to_lowercase();
        let gamemode = match mode_str.as_str() {
            "survival" | "s" | "0" => crate::types::player::Gamemode::Survival,
            "creative" | "c" | "1" => crate::types::player::Gamemode::Creative,
            "adventure" | "a" | "2" => crate::types::player::Gamemode::Adventure,
            "spectator" | "sp" | "3" => crate::types::player::Gamemode::Spectator,
            _ => {
                invoker
                    .send_message(format!("{}Unknown gamemode: {}", RED, mode_str))
                    .await?;
                return Ok(());
            }
        };

        if let CommandInvoker::Player { player } = invoker {
            if player.uuid == uuid {
                player.set_gamemode(gamemode).await?;

                invoker
                    .send_message(format!("{}Set your gamemode to {}", GREEN, mode_str))
                    .await?;

                return Ok(());
            }
        }

        let player = {
            let players = PLAYERS.read().await;
            players.get(&uuid).cloned()
        };

        let Some(player) = player else {
            invoker
                .send_message(format!("{}Player not found: {}", RED, args[0]))
                .await?;
            return Ok(());
        };

        {
            let mut player = player.lock().await;
            player.set_gamemode(gamemode).await?;
        }

        invoker
            .send_message(format!(
                "{}Set {}'s gamemode to {}",
                GREEN, args[0], mode_str
            ))
            .await
    })
}
