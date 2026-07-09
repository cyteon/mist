use std::pin::Pin;

use crate::server::commands::CommandInvoker;
use crate::server::state::play::{NAME_TO_UUID, PLAYERS};
use crate::types::colors::{GREEN, RED};

pub fn run<'a, 'b>(
    args: &'a [&'a str],
    invoker: &'a mut CommandInvoker<'b>,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        if args.len() < 3 {
            return invoker
                .send_message(format!("{}Usage: /give <player> <item> [amount]", RED))
                .await;
        }

        let player_name = args[0].to_lowercase();
        let uuid = {
            let map = NAME_TO_UUID.read().await;
            map.get(&player_name).cloned()
        };

        let Some(uuid) = uuid else {
            return invoker
                .send_message(format!("{}Player not found: {}", RED, player_name))
                .await;
        };

        let mut item_name = args[1].to_lowercase();

        if !item_name.starts_with("minecraft:") {
            item_name = format!("minecraft:{}", item_name);
        }

        let item_id = crate::types::items::get_item_id(&item_name);

        let amount: i32 = match args[2].parse() {
            Ok(num) => num,
            Err(_) => {
                invoker
                    .send_message(format!("{}Invalid amount: {}", RED, args[2]))
                    .await?;
                return Ok(());
            }
        };

        if let CommandInvoker::Player { player } = invoker {
            if player.uuid == uuid {
                player.give_item(item_id, amount).await?;

                invoker
                    .send_message(format!("{}Gave you {} of {}", GREEN, amount, item_name))
                    .await?;

                return Ok(());
            }
        }

        let player = {
            let players = PLAYERS.read().await;
            players.get(&uuid).cloned()
        };

        let Some(player) = player else {
            return invoker
                .send_message(format!("{}Player not found: {}", RED, player_name))
                .await;
        };

        {
            let mut player = player.lock().await;

            player.give_item(item_id, amount).await?;
        }

        invoker
            .send_message(format!(
                "{}Gave {} {} of {}",
                GREEN, player_name, amount, item_name
            ))
            .await
    })
}
