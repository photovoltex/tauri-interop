#[tauri_interop::conditional_use]
use crate::model::{TestState, TestStateEmit};
#[tauri_interop::conditional_use]
use tauri::{Manager, Window};

#[tauri_interop::conditional_command]
pub fn empty_invoke() {}

#[tauri_interop::conditional_command]
fn invoke_arguments(_string_to_string: ::std::string::String) {}

#[tauri_interop::conditional_command]
pub fn invoke_with_return(window: Window) -> String {
    window
        .windows()
        .into_keys()
        .intersperse(String::from(","))
        .collect()
}

#[tauri_interop::conditional_command]
pub fn invoke_with_return_vec() -> Vec<String> {
    vec![]
}

#[tauri_interop::conditional_command]
pub fn invoke_with_window_as_argument() -> i32 {
    420
}

#[tauri_interop::conditional_command]
pub fn emit(handle: tauri::AppHandle) {
    log::info!("echo cmd received");

    let test_state = TestState {
        echo: String::from("value"),
        foo: 420,
        bar: false,
    };

    test_state.emit(&handle, TestStateEmit::Echo).unwrap();
    test_state.emit(&handle, TestStateEmit::Foo).unwrap();
    test_state.emit(&handle, TestStateEmit::Bar).unwrap();
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

tauri_interop::collect_handlers!();
