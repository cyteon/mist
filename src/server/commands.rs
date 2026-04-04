use crate::types::colors::{GREEN, RED, YELLOW};
use crate::types::player::Player;

pub async fn handle_command(command: String, player: &mut Player) -> anyhow::Result<()> {
    let command_parts = command.split_whitespace().collect::<Vec<&str>>();

    match command_parts[0] {
        "tps" => {
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
                .await?;
        }

        "version" => {
            player
                .send_system_message(format!(
                    "{}Mist Server v{}",
                    GREEN,
                    env!("CARGO_PKG_VERSION")
                ))
                .await?;
        }

        "give" => {
            if command_parts.len() < 3 {
                player
                    .send_system_message(format!("{}Usage: /give <item> <amount>", RED))
                    .await?;
                return Ok(());
            }

            let item_name = command_parts[1];
            let amount: i32 = match command_parts[2].parse() {
                Ok(num) => num,
                Err(_) => {
                    player
                        .send_system_message(format!("{}Invalid amount: {}", RED, command_parts[2]))
                        .await?;
                    return Ok(());
                }
            };

            player
                .send_system_message(format!("{}Giving you {} of {}", GREEN, amount, item_name))
                .await?;

            let item_id = crate::types::items::get_item_id(item_name);
            player.give_item(item_id, amount).await?;
        }

        "gamemode" => {
            if command_parts.len() < 2 {
                player
                    .send_system_message(format!("{}Usage: /gamemode <mode>", RED))
                    .await?;
                return Ok(());
            }

            let mode_str = command_parts[1].to_lowercase();
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
                .await?;
        }

        _ => {
            player
                .send_system_message(format!("{}Unknown command: /{}", RED, command_parts[0]))
                .await?;
        }
    }

    Ok(())
}
