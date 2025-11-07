use tokio::{task, try_join};

use fancy_log::{LogLevel, log};

pub async fn run() -> anyhow::Result<()> {
    log(LogLevel::Info, format!("Starting server on {}:{}", 
        crate::config::SERVER_CONFIG.host,
        crate::config::SERVER_CONFIG.port
    ).as_str());

    let listener_task = task::spawn(crate::server::listener::start_listener());
    let tick_task = task::spawn(crate::server::tick::start_tick_loop());

    let _ = try_join!(
        listener_task,
        tick_task
    )?;

    Ok(())
}