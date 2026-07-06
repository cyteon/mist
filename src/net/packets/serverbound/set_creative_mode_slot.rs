use tokio::io::AsyncReadExt;

use crate::log::{self, LogLevel};
use crate::net::codec::read_slot;
use crate::types::entity::spawn_item_drop;
use crate::types::player::{Gamemode, Player};

pub async fn read_set_creative_mode_slot<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    if player.gamemode != Gamemode::Creative {
        log::log(
            LogLevel::Warn,
            &format!(
                "Player {} tried to set creative mode slot while not in creative mode",
                player.username
            ),
        );

        return Ok(());
    }

    let slot = stream.read_i16().await?;
    let item = read_slot(stream).await?;

    if slot == -1 {
        let Some(item) = item else {
            return Ok(());
        };

        let yaw = player.yaw.to_radians();
        let pitch = player.pitch.to_radians();

        let vx = -yaw.sin() as f64 * pitch.cos() as f64 * 0.3;
        let vy = -pitch.sin() as f64 * 0.3 + 0.1;
        let vz = yaw.cos() as f64 * pitch.cos() as f64 * 0.3;

        spawn_item_drop(
            item,
            Some(player.uuid.clone()),
            (player.x, player.y + 1.3, player.z),
            (vx, vy, vz),
        );

        return Ok(());
    }

    player.inventory[slot as usize] = item;

    if let Some(item) = item {
        log::log(
            LogLevel::Debug,
            &format!(
                "Player {} set creative mode slot {} to item {:?}",
                player.username, slot, item
            ),
        );
    }

    Ok(())
}
