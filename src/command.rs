#[cfg(not(target_family = "wasm"))]
pub use type_aliases::*;

/// wasm bindings for tauri's provided js functions (target: `wasm` or feat: `wasm`)
#[cfg(any(target_family = "wasm", feature = "wasm"))]
pub mod bindings;

#[cfg(not(target_family = "wasm"))]
mod type_aliases {
    /// Type alias to easier identify [tauri::State] via [tauri_interop_macro::command] macro
    pub type TauriState<'r, T> = tauri::State<'r, T>;

    /// Type alias to easier identify [tauri::Window] via [tauri_interop_macro::command] macro
    pub type TauriWindow = tauri::Window;

    /// Type alias to easier identify [tauri::AppHandle] via [tauri_interop_macro::command] macro
    pub type TauriAppHandle = tauri::AppHandle;
}
