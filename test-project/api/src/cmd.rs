tauri_interop::host_usage! {
    // usually u don't need to exclude the crates inside the api,
    // but when the type is removed because it is wrapped in a State,
    // it produced a warning... and we don't like warnings, so we exclude it
    use crate::model::TestState;
    | use std::sync::RwLock;
    | use tauri_interop::command::{TauriAppHandle, TauriState};
}

#[tauri_interop::command]
pub fn empty_invoke() {}

#[tauri_interop::command]
pub fn underscore_invoke(_invoke: u8) {}

#[tauri_interop::command]
pub async fn await_heavy_computing() {
    std::thread::sleep(std::time::Duration::from_millis(5000))
}

#[tauri_interop::command]
pub fn greet(name_to_greet: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name_to_greet)
}

#[tauri_interop::command]
pub fn invoke_with_return(window: tauri::WebviewWindow) -> String {
    use tauri::Manager;

    window
        .webview_windows()
        .into_keys()
        .intersperse(String::from(","))
        .collect()
}

#[tauri_interop::command]
pub fn invoke_with_return_vec() -> Vec<i32> {
    vec![69, 420]
}

#[tauri_interop::command]
pub fn result_test(switch_on: bool) -> Result<i32, String> {
    switch_on.then_some(69).ok_or(String::from("oh nyo"))
}

#[tauri_interop::command]
pub fn emit(state: TauriState<RwLock<TestState>>, handle: TauriAppHandle) {
    use tauri_interop::event::Emit;
    // newly generated mod, renamed to test_mod, default for TestState is test_state
    use crate::model::test_mod;

    log::info!("emit cmd received");

    let mut state = state.write().unwrap();

    let bar_value = !state.bar;
    let foo_value = if state.bar { "bar" } else { "foo" };

    state
        .update::<test_mod::FFoo>(&handle, foo_value.into())
        .unwrap();
    state.update::<test_mod::FBar>(&handle, bar_value).unwrap();
}

tauri_interop::collect_commands!();
