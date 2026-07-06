use std::pin::Pin;

use crate::server::commands::CommandInvoker;
use crate::types::player::broadcast_system_message;

pub fn run<'a, 'b>(
    args: &'a [&'a str],
    invoker: &'a mut CommandInvoker<'b>,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let message = args.join(" ");

        match invoker {
            CommandInvoker::Console => {
                broadcast_system_message(format!("[server] {}§f", message)).await?;
            }

            CommandInvoker::Player { player } => {
                broadcast_system_message(format!(
                    "[{}] {}§f",
                    player.username.to_lowercase(),
                    message
                ))
                .await?;
            }
        }

        Ok(())
    })
}
