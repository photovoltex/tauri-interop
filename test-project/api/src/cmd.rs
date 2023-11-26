#[tauri_interop::host_usage]
use crate::model::{TestState, TestStateEmit};

#[tauri_interop::command]
pub fn empty_invoke() {}

#[tauri_interop::command]
fn greet(name_to_greet: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name_to_greet)
}

#[tauri_interop::command]
pub fn invoke_with_return(window: tauri::Window) -> String {
    use tauri::Manager;

    window
        .windows()
        .into_keys()
        .intersperse(String::from(","))
        .collect()
}

#[tauri_interop::command]
pub fn invoke_with_return_vec() -> Vec<i32> {
    vec![69, 420]
}

#[tauri_interop::command]
pub fn result_test() -> Result<i32, String> {
    Ok(69)
}

#[tauri_interop::command]
pub fn emit(handle: tauri::AppHandle) {
    log::info!("emit cmd received");

    let mut test_state = TestState::default();
    test_state.update_foo(&handle, "update from backend".into()).unwrap();

    test_state.emit(&handle, TestStateEmit::Bar).unwrap();
    test_state.bar = true;
    test_state.emit(&handle, TestStateEmit::Bar).unwrap();
}

#[cfg(feature = "broken")]
pub mod broken {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum State {
        Test,
    }

    #[allow(clippy::result_unit_err)]
    /// currently this doesn't work cause of the way tauri::{AppHandel, State, Window} are filtered
    #[tauri_interop::conditional_command]
    pub fn invoke_result_tauri(_state: State) -> Result<(), ()> {
        Ok(())
    }
}

tauri_interop::collect_commands!();
