use std::pin::Pin;

use crate::server::commands::CommandInvoker;
use crate::server::state::play::{NAME_TO_UUID, PLAYERS};
use crate::types::colors::{GREEN, YELLOW};

pub fn run<'a, 'b>(
    args: &'a [&'a str],
    invoker: &'a mut CommandInvoker<'b>,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        if args.len() < 1 {
            invoker
                .send_message(format!("{}Usage: /op <player>", GREEN))
                .await?;
            return Ok(());
        }

        let target_username = args[0];

        if let CommandInvoker::Player { player } = invoker {
            if target_username.to_lowercase() == player.username.to_lowercase() {
                player
                    .send_system_message(format!("{}You cannot op yourself", YELLOW))
                    .await?;

                return Ok(());
            }
        }

        let uuid = {
            let map = NAME_TO_UUID.read().await;
            map.get(&target_username.to_lowercase()).cloned()
        };

        let Some(uuid) = uuid else {
            invoker
                .send_message(format!("{}Player not found: {}", YELLOW, target_username))
                .await?;

            return Ok(());
        };

        let target = {
            let players = PLAYERS.read().await;
            players.get(&uuid).cloned()
        };

        let Some(target) = target else {
            invoker
                .send_message(format!("{}Player not found: {}", YELLOW, target_username))
                .await?;

            return Ok(());
        };

        {
            let mut target = target.lock().await;
            target.set_op(true).await?;

            target
                .send_system_message(format!("{}You are now an operator", GREEN))
                .await?;
        }

        invoker
            .send_message(format!(
                "{}You have made {} an operator",
                GREEN, target_username
            ))
            .await?;

        Ok(())
    })
}
