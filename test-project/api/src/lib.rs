#![allow(clippy::disallowed_names)]
#![feature(iter_intersperse)]
#![feature(proc_macro_hygiene)]

#[cfg(target_family = "wasm")]
pub use tauri_interop::*;

#[tauri_interop::commands]
pub mod cmd;

pub mod model;

tauri_interop::combine_handlers!(
    cmd,
    model::other_cmd,
    model::test_mod,
    model::NamingTestEnumField,
    model::naming_test_default
);
