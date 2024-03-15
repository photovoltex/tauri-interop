#[cfg(not(target_family = "wasm"))]
#[doc(cfg(not(target_family = "wasm")))]
pub use type_aliases::*;

/// wasm bindings for tauri's provided js functions
#[cfg(any(target_family = "wasm", doc))]
#[doc(cfg(target_family = "wasm"))]
pub mod bindings;

#[cfg(not(target_family = "wasm"))]
#[doc(cfg(not(target_family = "wasm")))]
mod type_aliases;
