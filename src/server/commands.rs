use crate::types::player::Player;
use crate::types::colors::{GREEN, RED};

pub async fn handle_command(command: String, player: &mut Player) -> anyhow::Result<()> {
    let command_parts = command.split_whitespace().collect::<Vec<&str>>();

    match command_parts[0] {
        "tps" => {
            let tps = crate::server::tick::TPS_5S.load(std::sync::atomic::Ordering::Relaxed);
            player.send_system_message(format!("{}Current TPS: {}", GREEN, tps)).await?;
        }

        "version" => {
            player.send_system_message(format!("{}Mist Server v{}", GREEN, env!("CARGO_PKG_VERSION"))).await?;
        }

        "give" => {
            if command_parts.len() < 3 {
                player.send_system_message(format!("{}Usage: /give <item> <amount>", RED)).await?;
                return Ok(());
            }

            let item_name = command_parts[1];
            let amount: i32 = match command_parts[2].parse() {
                Ok(num) => num,
                Err(_) => {
                    player.send_system_message(format!("{}Invalid amount: {}", RED, command_parts[2])).await?;
                    return Ok(());
                }
            };

            player.send_system_message(format!("{}Giving you {} of {}", GREEN, amount, item_name)).await?;

            let item_id = crate::types::items::get_item_id(item_name);
            player.give_item(item_id, amount).await?;
        }

        _ => {
            player.send_system_message(format!("{}Unknown command: /{}", RED, command_parts[0])).await?;
        }
    }

    Ok(())
}