use crate::net::codec::{read_hashed_slot, read_var};
use crate::types::player::{Player, WindowType};
use crate::types::recipes::{check_2x2, check_3x3};
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

    match &mut player.current_window {
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

        Some(WindowType::CraftingTable(grid)) => {
            for _ in 0..changed_slots_len {
                let slot_index = stream.read_i16().await?;
                let item_stack = read_hashed_slot(stream).await?;

                if (0..=9).contains(&slot_index) {
                    grid[slot_index as usize] = item_stack.clone();
                } else if (10..=45).contains(&slot_index) {
                    player.inventory[slot_index as usize - 1] = item_stack.clone();
                }
            }

            let _carried_item = read_hashed_slot(stream).await?;

            let crafting_grid = [
                grid[1].as_ref().map(|s| s.item_id),
                grid[2].as_ref().map(|s| s.item_id),
                grid[3].as_ref().map(|s| s.item_id),
                grid[4].as_ref().map(|s| s.item_id),
                grid[5].as_ref().map(|s| s.item_id),
                grid[6].as_ref().map(|s| s.item_id),
                grid[7].as_ref().map(|s| s.item_id),
                grid[8].as_ref().map(|s| s.item_id),
                grid[9].as_ref().map(|s| s.item_id),
            ];

            grid[0] = check_3x3(&crafting_grid)
                .map(|(id, count)| crate::types::items::ItemStack { item_id: id, count });

            let tx = crate::server::conn::PLAYER_SOCKET_MAP
                .read()
                .await
                .get(&player.uuid)
                .unwrap()
                .clone();

            let mut buffer = Vec::new();

            crate::net::packets::clientbound::container_set_slot::send_container_set_slot(
                &mut buffer,
                player.window_id as u8,
                0,
                grid[0].clone(),
            )
            .await?;

            let _ = tx.send(buffer);
        }
    }

    Ok(())
}
