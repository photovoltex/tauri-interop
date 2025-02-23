# Tauri-Interop

[![Latest version](https://img.shields.io/crates/v/tauri-interop.svg)](https://crates.io/crates/tauri-interop)
[![Documentation](https://docs.rs/tauri-interop/badge.svg)](https://docs.rs/tauri-interop)
![License](https://img.shields.io/crates/l/tauri-interop.svg)

This crate tries to provide a general more enjoyable experience for developing tauri apps with a rust frontend.

Writing an app in a single language gives us the option of building a common crate/module which connects the backend and
frontend. A common model itself can most of the time be easily compiled to both architectures (arch's) when the types
are compatible with both. The commands on the other hand don't have an option to be compiled to wasm. Which means they
need to be handled manually or be called via a wrapper/helper each time.

The crates therefore provides the following features:

- generate a wasm function out of the defined tauri-command
- collect and register all defined tauri-commands
- QOL-macros to exclude multiple imports in wasm or the host architecture
- easier usage of [tauri's event feature](https://tauri.app/v1/guides/features/events/)

## Compatability

- toolchain: `nightly`

| tauri-interop | tauri | leptos |
|---------------|-------|--------|
| <= 2.1.6      | 1.6   | 0.6    |
| current       | 2     | 0.7    |

### Note

The library uses a resolver 2 features to allow easy inclusion without configuration. When working with virtual
workspaces the resolver defaults to 1. In that case it is required to set the resolver manually to version 2,  
otherwise
the [target specific compilation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies)
will not resolve correctly. When the wrong resolver is used, an error should state that the `Listen` trait is missing.

