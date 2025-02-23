use api::model::TestState;
use std::sync::RwLock;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(api::get_all_handlers())
        .setup(move |app| {
            let main_window = app.handle().get_webview_window("main").unwrap();

            // debugging: always open dev tools on launch
            main_window.open_devtools();

            let test_state = RwLock::new(TestState::default());
            app.manage(test_state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
