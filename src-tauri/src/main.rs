use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .invoke_handler(api::cmd::get_handlers())
        .setup(move |app| {
                let main_window = app
                    .handle()
                    .get_window("main")
                    .unwrap();

                // debugging: always open dev tools on launch
                main_window.open_devtools();

                Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
