#[cfg(not(target_family = "wasm"))]
pub use type_aliases::*;

/// wasm bindings for tauri's provided js functions (target: `wasm`)
#[cfg(any(target_family = "wasm", doc))]
pub mod bindings;

#[cfg(not(target_family = "wasm"))]
mod type_aliases;
