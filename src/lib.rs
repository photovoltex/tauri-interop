//! Tauri-Interop is a library that provides macros to improve developing tauri apps with a rust
//! frontend by generating frontend implementation out of the backend definitions.
//!
//! The main macros intended to be used are:
//! - [macro@command], which is intended to be used as replacement to [macro@tauri::command]
//! - [macro@Event], that provides an easier usage of the [Events feature of tauri](https://v2.tauri.app/develop/calling-frontend/)
//!     - derives [event::Listen] when compiling to wasm and [event::Emit] otherwise
//!
//! Additionally, some QOL macros ([host_usage] and [wasm_usage]) are provided that
//! reduce some drawbacks when simultaneously compiling to wasm and the host architecture.
//!
//! ### Explanation and Examples
//!
//! Detail explanations and example can be found on the respected traits or macros. Some
//! examples are ignored because they are only valid when compiling to wasm.
//!
//! ### Note
//!
//! The library uses resolver 2 features to allow easy inclusion without configuration. When working
//! with virtual workspaces the resolver defaults to 1 in which case it is required to set the
//! resolver manually to version 2, otherwise the [target specific compilation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies)
//! will not resolve correctly. When the wrong resolver is used, an error should state that the
//! [event::Listen] trait is missing.

#![feature(trait_alias)]
#![feature(doc_cfg)]
#![warn(missing_docs)]

#[cfg(any(target_family = "wasm", doc))]
#[doc(cfg(target_family = "wasm"))]
pub use tauri_interop_macro::binding;
pub use tauri_interop_macro::*;
#[cfg(not(target_family = "wasm"))]
#[doc(cfg(not(target_family = "wasm")))]
pub use tauri_interop_macro::{collect_commands, combine_handlers, commands};
#[cfg(feature = "event")]
#[doc(cfg(feature = "event"))]
pub use tauri_interop_macro::{Emit, EmitField, Event, Listen, ListenField};

/// wrapped bindings for easier use in the generated wasm commands
pub mod command;
/// event traits and overall logic for event emitting and listening
#[cfg(feature = "event")]
#[doc(cfg(feature = "event"))]
pub mod event;

// re-exported crates

#[doc(hidden)]
pub use log;

#[doc(hidden)]
pub use serde;
