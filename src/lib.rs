#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![feature(trait_alias)]

pub use tauri_interop_macro::*;

/// wrapped bindings for easier use in the generated wasm commands
pub mod command;
/// event traits and overall logic for event emitting and listening (feat: `event`)
#[cfg(feature = "event")]
pub mod event;
