use tokio::io::AsyncReadExt;

use crate::{
    log::{self, LogLevel},
    net::{
        codec::{read_position, read_var},
        packets::clientbound::{
            block_action::send_block_action, container_set_content::send_container_set_content,
            open_screen,
        },
    },
    types::{
        block_entities::{BlockEntityData, get_block_entity},
        blocks::{self, block_by_state_id, compute_overrides, resolve_state},
        items::ItemStack,
        player::{Player, broadcast_packet},
    },
    world::get_region,
};

pub async fn read_use_item_on<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let _hand = read_var(stream).await?;
    let (x, y, z) = read_position(stream).await?;

    let face = stream.read_u8().await?;

    let _cursor_x = stream.read_f32().await?;
    let cursor_y = stream.read_f32().await?;
    let _cursor_z = stream.read_f32().await?;

    let _inside_block = stream.read_u8().await?;
    let _world_border_hit = stream.read_u8().await?;
    let sequence = read_var(stream).await?;

    let (mut bx, mut by, mut bz) = (x, y, z);

    match face {
        0 => by -= 1,
        1 => by += 1,
        2 => bz -= 1,
        3 => bz += 1,
        4 => bx -= 1,
        5 => bx += 1,
        _ => {}
    }

    let chunk_pos = (bx.div_euclid(16), bz.div_euclid(16));
    let region_pos = (chunk_pos.0.div_euclid(32), chunk_pos.1.div_euclid(32));

    let region = get_region(region_pos.0, region_pos.1).await;
    let mut region_lock = region.lock().await;

    if let Some(chunk) = region_lock.get_chunk(chunk_pos.0, chunk_pos.1)
        && !player.movement.sneaking
    {
        let block_id = chunk.get_block((x & 15) as u8, y as i32, (z & 15) as u8);
        let default_block_id = block_by_state_id(block_id)
            .map(|block| block.default_state)
            .unwrap_or(0);

        match default_block_id {
            blocks::CRAFTING_TABLE => {
                let mut buffer = Vec::new();

                open_screen::send_open_screen(
                    &mut buffer,
                    player
                        .new_window_id(crate::types::player::WindowType::CraftingTable([None; 10])),
                    open_screen::WindowType::CraftingTable,
                    "Crafting",
                )
                .await?;

                player.send_packet(buffer).await?;

                return Ok(());
            }

            blocks::CHEST => {
                let Some(BlockEntityData::Chest { items, viewers }) =
                    chunk.block_entities.get_mut(&((x & 15), y, (z & 15)))
                else {
                    log::log(
                        LogLevel::Warn,
                        &format!(
                            "Block entity data for chest at ({}, {}, {}) not found",
                            x & 15,
                            y,
                            z & 15
                        ),
                    );

                    return Ok(());
                };

                let mut buffer = Vec::new();
                let window_id = player
                    .new_window_id(crate::types::player::WindowType::Chest { cords: (x, y, z) });

                open_screen::send_open_screen(
                    &mut buffer,
                    window_id,
                    open_screen::WindowType::Chest,
                    "Chest",
                )
                .await?;
                player.send_packet(buffer).await?;

                let mut chest_container: [Option<ItemStack>; 63] = [None; 63];

                for (i, item) in items.iter().enumerate() {
                    chest_container[i] = item.clone();
                }

                for i in 9..=44 {
                    chest_container[i + 18] = player.inventory[i].clone();
                }

                let mut buffer = Vec::new();
                send_container_set_content(
                    &mut buffer,
                    window_id as u8,
                    chest_container.to_vec(),
                    None,
                )
                .await?;
                player.send_packet(buffer).await?;

                viewers.push(player.uuid.clone());

                let mut buffer = Vec::new();
                send_block_action(&mut buffer, (x, y, z), 1, viewers.len() as u8).await?;
                broadcast_packet(buffer, (x as f64, y as f64, z as f64), None).await?;

                return Ok(());
            }

            blocks::FURNACE => {
                let Some(BlockEntityData::Furnace {
                    input,
                    output,
                    fuel,
                    ..
                }) = chunk.block_entities.get_mut(&((x & 15), y, (z & 15)))
                else {
                    log::log(
                        LogLevel::Warn,
                        &format!(
                            "Block entity data for furnace at ({}, {}, {}) not found",
                            x & 15,
                            y,
                            z & 15
                        ),
                    );

                    return Ok(());
                };

                let mut buffer = Vec::new();

                let window_id = player
                    .new_window_id(crate::types::player::WindowType::Furnace { cords: (x, y, z) });

                open_screen::send_open_screen(
                    &mut buffer,
                    window_id,
                    open_screen::WindowType::Furnace,
                    "Furnace",
                )
                .await?;

                player.send_packet(buffer).await?;

                let mut furnace_container: [Option<ItemStack>; 39] = [None; 39];

                furnace_container[0] = input.clone();
                furnace_container[1] = fuel.clone();
                furnace_container[2] = output.clone();

                for i in 9..=44 {
                    furnace_container[i - 6] = player.inventory[i].clone();
                }

                let mut buffer = Vec::new();
                send_container_set_content(
                    &mut buffer,
                    window_id as u8,
                    furnace_container.to_vec(),
                    None,
                )
                .await?;
                player.send_packet(buffer).await?;

                return Ok(());
            }

            _ => {}
        }
    }

    let default_block_id =
        if let Some(Some(item_stack)) = player.inventory.get(player.current_slot as usize + 36) {
            if let Some(block_id) = crate::types::items::item_to_block(item_stack.item_id) {
                block_id
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

    if let Some(chunk) = region_lock
        .chunks
        .iter_mut()
        .find(|chunk| chunk.x == chunk_pos.0 && chunk.z == chunk_pos.1)
    {
        let block =
            block_by_state_id(default_block_id).expect("Block ID not found in block registry");

        let past_block_id = chunk.get_block((bx & 15) as u8, by, (bz & 15) as u8);
        let overrides = compute_overrides(past_block_id, &block, face, player.yaw, cursor_y);
        let block_id = resolve_state(block, overrides) as u16;

        chunk.set_block((bx & 15) as u8, by, (bz & 15) as u8, block_id);

        if let Some(be) = get_block_entity(block_id) {
            chunk.block_entities.insert((bx & 15, by, bz & 15), be);
        }

        if player.gamemode as u8 != 1 {
            player.inventory[player.current_slot as usize + 36]
                .as_mut()
                .unwrap()
                .count -= 1;

            if player.inventory[player.current_slot as usize + 36]
                .as_ref()
                .unwrap()
                .count
                == 0
            {
                player.inventory[player.current_slot as usize + 36] = None;
            }
        }

        let mut buffer = Vec::new();
        crate::net::packets::clientbound::block_changed_ack::send_block_changed_ack(
            &mut buffer,
            sequence as i32,
        )
        .await?;

        player.send_packet(buffer).await?;

        crate::net::packets::clientbound::block_update::broadcast_block_update(
            bx,
            by,
            bz,
            block_id as i32,
        )
        .await?;
    }

    Ok(())
}
