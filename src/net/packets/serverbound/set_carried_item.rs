use crate::{
    log::{self, LogLevel},
    types::player::Player,
};
use tokio::io::AsyncReadExt;

pub async fn read_set_carried_item<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    player: &mut Player,
) -> anyhow::Result<()> {
    let slot = stream.read_i16().await?;

    if !(0..=8).contains(&slot) {
        log::log(
            LogLevel::Warn,
            &format!(
                "Player {} tried to set carried item to invalid slot {}",
                player.username, slot
            ),
        );
    }

    player.current_slot = slot;

    Ok(())
}
