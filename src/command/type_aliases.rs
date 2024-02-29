use tauri::{AppHandle, State, Window};

#[allow(unused_imports)]
use tauri_interop_macro::command;

/// Type alias to easier identify [State] via [command] macro
pub type TauriState<'r, T> = State<'r, T>;

/// Type alias to easier identify [Window] via [command] macro
pub type TauriWindow = Window;

/// Type alias to easier identify [AppHandle] via [command] macro
pub type TauriAppHandle = AppHandle;
