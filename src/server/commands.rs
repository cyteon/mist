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

        _ => {
            player.send_system_message(format!("{}Unknown command: /{}", RED, command_parts[0])).await?;
        }
    }

    Ok(())
}