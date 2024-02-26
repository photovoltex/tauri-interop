use tauri::{AppHandle, State, Window};

/// Type alias to easier identify [State] via [tauri_interop_macro::command] macro
pub type TauriState<'r, T> = State<'r, T>;

/// Type alias to easier identify [Window] via [tauri_interop_macro::command] macro
pub type TauriWindow = Window;

/// Type alias to easier identify [AppHandle] via [tauri_interop_macro::command] macro
pub type TauriAppHandle = AppHandle;
