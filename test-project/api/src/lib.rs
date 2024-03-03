#![allow(clippy::disallowed_names)]
#![feature(iter_intersperse)]

pub mod command;
pub mod model;

#[cfg(target_family = "wasm")]
pub use tauri_interop::*;
