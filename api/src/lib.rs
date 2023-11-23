#![feature(iter_intersperse)]

pub mod cmd;
pub mod model;

#[cfg(target_family = "wasm")]
pub use tauri_interop::*;
