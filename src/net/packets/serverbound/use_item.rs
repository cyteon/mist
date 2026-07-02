use tokio::io::AsyncReadExt;

use crate::{
    net::codec::read_var,
    types::{items::get_food_data, player::Player},
};

pub async fn read_use_item<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let _hand = read_var(stream).await?;
    let _sequence = read_var(stream).await?;

    let current_item = player.inventory[player.current_slot as usize + 36];

    match current_item {
        Some(item) => {
            if let Some(food_data) = get_food_data(item.item_id) {
                if food_data.2 || player.hunger < 20 {
                    player.eating_ticks_left = 32;
                    player.eating_slot = player.current_slot;
                }
            }
        }

        None => {}
    }

    Ok(())
}
