use std::pin::Pin;

use crate::server::commands::CommandInvoker;
use crate::types::colors::GREEN;

pub fn run<'a, 'b>(
    _args: &'a [&'a str],
    invoker: &'a mut CommandInvoker<'b>,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        invoker
            .send_message(format!(
                "{}Mist Server v{}",
                GREEN,
                env!("CARGO_PKG_VERSION"),
            ))
            .await
    })
}
