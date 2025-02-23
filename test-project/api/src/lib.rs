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
    // the enum isn't registered as a state in tauri, so registering it wouldn't work anyways
    // model::NamingTestEnumField,
    model::naming_test_default
);
