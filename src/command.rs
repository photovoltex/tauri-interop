#[cfg(not(target_family = "wasm"))]
pub use type_aliases::*;

/// wasm bindings for tauri's provided js functions (target: `wasm` or feat: `wasm`)
#[cfg(any(target_family = "wasm", feature = "_wasm"))]
pub mod bindings;

#[cfg(not(target_family = "wasm"))]
mod type_aliases;
