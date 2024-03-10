# Tauri-Interop

[![Latest version](https://img.shields.io/crates/v/tauri-interop.svg)](https://crates.io/crates/tauri-interop)
[![Documentation](https://docs.rs/tauri-interop/badge.svg)](https://docs.rs/tauri-interop)
![License](https://img.shields.io/crates/l/tauri-interop.svg)

This crate tries to provide a general more enjoyable experience for developing tauri apps with a rust frontend.
> tbf it is a saner approach to write the app in a mix of js + rust, because the frameworks are more mature, there are
> way more devs who have experience with js and their respective frameworks etc...
> 
> but tbh... just because something is saner, doesn't stop us from doing things differently ^ヮ^

Writing an app in a single language gives us the option of building a common crate/module which connects the backend and 
frontend. A common model itself can most of the time be easily compiled to both architectures (arch's) when the types 
are compatible with both. The commands on the other hand don't have an option to be compiled to wasm. Which means they
need to be handled manually or be called via a wrapper/helper each time.

Repeating the implementation and handling for a function that is already defined properly seems to be a waste of time.
For that reason this crate provides the `tauri_interop::command` macro. This macro is explained in detail in the 
[command representation](#command-representation-hostwasm) section. This new macro provides the option to invoke the 
command in wasm and by therefore call the defined command in tauri. On the other side, when compiling for tauri in addition 
to the tauri logic, the macro provides the option to collect all commands in a single file via the invocation of the 
`tauri_interop::collect_commands` macro at the end of the file (see [command](#command-frontend--backend-communication)).

In addition, some quality-of-life macros are provided to ease some inconveniences when compiling to multiple arch's. See
the [QOL](#qol-macros) section.

**Feature `event`**:

Tauri has an [event](https://tauri.app/v1/guides/features/events) mechanic which allows the tauri side to communicate with
the frontend. The usage is not as intuitive and has to some inconveniences that make it quite hard to recommend. To 
improve the usage, this crate provides the derive-marcos `Event`, `Emit` and `Listen`. The `Event` macro is just a 
conditional wrapper that expands to `Emit` for the tauri compilation and `Listen` for the wasm compilation. It is 
the intended way to use this feature. The usage is explained in the documentation of the `Event` macro. 
section.

### QOL macros

This crate also adds some quality-of-life macros. These are intended to ease the drawbacks of compiling to
multiple architectures.

#### Conditional `use`
Because most crates are not intended to be compiled to wasm and most wasm crates are not intended to be compiled to
the host-triplet they have to be excluded in each others compile process. The usual process to exclude uses for a certain
architecture would look something like this:

```rust
#[cfg(not(target_family = "wasm"))]
use tauri::AppHandle;

#[tauri_interop::command]
pub fn empty_invoke(_handle: AppHandle) {}
```

**General usage:**

With the help of `tauri_interop::host_usage!()` and `tauri_interop::wasm_usage!()` we don't need to remember which
attribute we have to add and can just convert the above to the following:

```rust
tauri_interop::host_usage! {
    use tauri::AppHandle;
}

#[tauri_interop::command]
pub fn empty_invoke(_handle: AppHandle) {}
```

**Multiple `use` usage:**

When multiple `use` should be excluded, they need to be separated by a single pipe (`|`). For example:

```rust
tauri_interop::host_usage! {
    use tauri::State;
    | use std::sync::RwLock; 
}

#[tauri_interop::command]
pub fn empty_invoke(_state: State<RwLock<String>>) {}
```
