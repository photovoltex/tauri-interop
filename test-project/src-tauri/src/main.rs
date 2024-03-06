use std::sync::RwLock;

use api::model::TestState;
use tauri::Manager;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .unwrap();

    tauri::Builder::default()
        .invoke_handler(api::get_all_handlers())
        .setup(move |app| {
            let main_window = app.handle().get_window("main").unwrap();

            // debugging: always open dev tools on launch
            main_window.open_devtools();

            let test_state = RwLock::new(TestState::default());
            app.manage(test_state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
