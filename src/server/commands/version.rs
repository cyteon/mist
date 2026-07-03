use std::pin::Pin;

use crate::types::colors::GREEN;
use crate::types::player::Player;

pub fn run<'a>(
    _args: &'a [&'a str],
    player: &'a mut Player,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        player
            .send_system_message(format!(
                "{}Mist Server v{}",
                GREEN,
                env!("CARGO_PKG_VERSION"),
            ))
            .await
    })
}
