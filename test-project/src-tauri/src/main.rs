fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .unwrap();

    src_tauri::run()
}
