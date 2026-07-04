use std::pin::Pin;

use crate::server::commands::CommandInvoker;
use crate::types::colors::{GREEN, RED};
use crate::types::player::Player;

pub fn run<'a, 'b>(
    args: &'a [&'a str],
    invoker: &'a mut CommandInvoker<'b>,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        // TODO: make it /give <player> <item> <amount>
        let CommandInvoker::Player { player } = invoker else {
            invoker
                .send_message(format!("{}This command can only be ran by a player", RED))
                .await?;
            return Ok(());
        };

        if args.len() < 2 {
            return player
                .send_system_message(format!("{}Usage: /give <item> <amount>", RED))
                .await;
        }

        let mut item_name = args[0].to_lowercase();

        if !item_name.starts_with("minecraft:") {
            item_name = format!("minecraft:{}", item_name);
        }

        let amount: i32 = match args[1].parse() {
            Ok(num) => num,
            Err(_) => {
                player
                    .send_system_message(format!("{}Invalid amount: {}", RED, args[1]))
                    .await?;
                return Ok(());
            }
        };

        player
            .send_system_message(format!("{}Giving you {} of {}", GREEN, amount, item_name))
            .await?;

        let item_id = crate::types::items::get_item_id(&item_name);
        player.give_item(item_id, amount).await
    })
}
