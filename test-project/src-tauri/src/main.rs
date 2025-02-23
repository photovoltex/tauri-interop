fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .unwrap();

    app_lib::run()
}
