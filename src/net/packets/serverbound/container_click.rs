use crate::net::codec::{read_hashed_slot, read_var};
use crate::types::player::{Player, WindowType};
use crate::types::recipes::check_2x2;
use tokio::io::AsyncReadExt;

pub async fn read_container_click<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let window_id = read_var(stream).await?;
    let _state_id = read_var(stream).await?;

    let _slot = stream.read_i16().await?;
    let _button = stream.read_u8().await?;

    let _mode = read_var(stream).await?;

    let changed_slots_len = read_var(stream).await?;

    match player.current_window {
        None => {
            // todo: the client is a liar, dont trust the client, never trust the client, the client is a liar
            // only trust the server, the server dosent lie, trust the server
            for _ in 0..changed_slots_len {
                let slot_index = stream.read_i16().await?;
                let item_stack = read_hashed_slot(stream).await?;

                if window_id == 0 && (0..46).contains(&slot_index) {
                    player.inventory[slot_index as usize] = item_stack.clone();
                }

                println!("Changed slot {} to {:?}", slot_index, item_stack);
            }

            let carried_item = read_hashed_slot(stream).await?;
            player.carried_item = carried_item.clone();

            let crafting_grid = [
                player.inventory[1].as_ref().map(|s| s.item_id),
                player.inventory[2].as_ref().map(|s| s.item_id),
                player.inventory[3].as_ref().map(|s| s.item_id),
                player.inventory[4].as_ref().map(|s| s.item_id),
            ];

            player.inventory[0] = check_2x2(&crafting_grid)
                .map(|(id, count)| crate::types::items::ItemStack { item_id: id, count });

            player.sync_player_inventory().await?;
        }

        Some(WindowType::CraftingTable(grid)) => {}
    }

    Ok(())
}
