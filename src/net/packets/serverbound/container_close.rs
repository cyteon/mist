use crate::net::codec::read_var;
use crate::net::packets::clientbound::block_action::send_block_action;
use crate::types::player::{Player, WindowType};
use tokio::io::AsyncReadExt;

pub async fn read_container_close<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let _window_id = read_var(stream).await?;

    match player.current_window {
        Some(window) => match window {
            WindowType::CraftingTable(grid) => {
                for i in 1..=9 {
                    if let Some(item_stack) = grid[i] {
                        player
                            .give_item(item_stack.item_id, item_stack.count as i32)
                            .await?;
                    }
                }
            }

            WindowType::Chest { cords, .. } => {
                let mut buffer = Vec::new();
                send_block_action(&mut buffer, cords, 1, 0).await?;
                player.send_packet(buffer).await?;
            }
        },

        None => {}
    }

    player.current_window = None;

    Ok(())
}
