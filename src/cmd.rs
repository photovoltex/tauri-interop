#[tauri_interop::conditional_use]
use tauri::Window;

#[tauri_interop::conditional_command]
pub fn empty_invoke() {}

#[tauri_interop::conditional_command]
async fn invoke_arguments(_string_to_string: ::std::string::String) {}

#[tauri_interop::conditional_command]
pub fn invoke_with_return() -> String {
    "test".to_string()
}

#[tauri_interop::conditional_command]
pub fn invoke_with_return_vec() -> Vec<String> {
    vec![]
}

#[tauri_interop::conditional_command]
pub fn invoke_with_window_as_argument(_window: Window) -> i32 {
    420
}

#[tauri_interop::conditional_command]
pub fn invoke_promise_with_app_handle_as_argument(_handle: tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(feature = "broken")]
pub mod broken {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum State {
        OwO,
    }
    
    #[allow(clippy::result_unit_err)]
    /// currently this doesn't work cause of the way tauri::{AppHandel, State, Window} are filtered
    #[tauri_interop::conditional_command]
    pub fn invoke_result_tauri(_state: State) -> Result<(), ()> {
        Ok(())
    }
}

tauri_interop::setup!();
