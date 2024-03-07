#![allow(clippy::disallowed_names)]
#![feature(iter_intersperse)]
#![feature(proc_macro_hygiene)]

#[tauri_interop::commands]
pub mod cmd;

pub mod model;

#[cfg(target_family = "wasm")]
pub use tauri_interop::*;

tauri_interop::combine_handlers!( cmd, model::other_cmd );
