use fancy_log::LogLevel;

pub fn ensure_save_folder() {
    std::fs::create_dir_all(crate::config::SERVER_CONFIG.world_name.clone()).unwrap();
}

pub fn save() {
    ensure_save_folder();

    fancy_log::log(LogLevel::Info, "Autosaving...");
}