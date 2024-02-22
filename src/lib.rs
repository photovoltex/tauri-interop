#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![feature(trait_alias)]

pub use tauri_interop_macro::*;

/// wasm bindings for tauri's provided js functions (target: `wasm` or feat: `wasm`)
#[cfg(any(target_family = "wasm", feature = "wasm"))]
pub mod bindings;
/// wrapped bindings for easier use in the generated wasm commands (target: `wasm` or feat: `wasm`)
#[cfg(any(target_family = "wasm", feature = "wasm"))]
pub mod command;
/// event traits and overall logic for event emitting and listening (feat: `event`)
#[cfg(feature = "event")]
pub mod event;
