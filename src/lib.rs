#![warn(missing_docs)]
#![feature(trait_alias)]

//! Tauri-Interop is a library that provides macros to improve developing tauri apps with a rust 
//! frontend by generating frontend implementation out of the backend definitions. 
//! 
//! The main macros intended to be used are:
//! - [macro@command], which is intended to be used as replacement to [macro@tauri::command]
//! - [macro@Event], that provides an easier usage of the [Events feature of tauri](https://tauri.app/v1/guides/features/events/)
//!     - derives [event::Listen] when compiling to wasm and [event::Emit] otherwise
//! 
//! Additionally, some QOL macros ([host_usage] and [wasm_usage]) are provided that 
//! reduce some drawbacks when simultaneously compiling to wasm and the host architecture.
//! 
//! ### Explanation and Examples
//! 
//! Detail explanations and example can be found on the respected traits or macros. Some 
//! examples are ignored because they are only valid when compiling to wasm.

pub use tauri_interop_macro::*;

/// wrapped bindings for easier use in the generated wasm commands
pub mod command;
/// event traits and overall logic for event emitting and listening (feat: `event`)
#[cfg(feature = "event")]
pub mod event;
