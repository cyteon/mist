use tokio::{task, try_join};
use fancy_log::{LogLevel, log};

use crate::server::save;

pub async fn run() -> anyhow::Result<()> {
    log(LogLevel::Info, format!("Starting server on {}:{}", 
        crate::config::SERVER_CONFIG.host,
        crate::config::SERVER_CONFIG.port
    ).as_str());

    crate::server::save::ensure_save_folders();

    if !save::exists("regions/0_0.mist_region") {
        crate::world::worldgen::initial_gen().await;
    } else {
        crate::server::save::load_world().await;
    }

    // server setup stuff goes here before listener activates

    let listener_task = task::spawn(crate::server::listener::start_listener());
    let tick_task = task::spawn(crate::server::tick::start_tick_loop());

    let _ = try_join!(
        listener_task,
        tick_task
    )?;

    Ok(())
}