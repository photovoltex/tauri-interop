# Tauri-Interop

[![Latest version](https://img.shields.io/crates/v/tauri-interop.svg)](https://crates.io/crates/tauri-interop)
[![Documentation](https://docs.rs/tauri-interop/badge.svg)](https://docs.rs/tauri-interop)
![License](https://img.shields.io/crates/l/tauri-interop.svg)

This crate tries to provide a general more enjoyable experience for developing tauri apps with a rust frontend.

Writing an app in a single language gives us the option of building a common crate which connects the host and
wasm target. A common model itself can most of the time be easily compiled to both architectures (arch's) when the types
are compatible with both. The commands on the other hand don't have an option to be compiled to wasm. Which means they
need to be handled manually or be called via a wrapper/helper each time.

The crates therefore provides the following features:

- generate a wasm function out of the defined tauri-command (`tauri_interop::command`)
- collect and register all defined tauri-commands (`tauri_interop::collect_commands`)
- QOL-macros to exclude multiple imports in wasm or the host architecture (`tauri_interop::{host_usage, wasm_usage}`)
- easier usage of [tauri's event feature](https://tauri.app/v1/guides/features/events/) (feature: `event`)

### Commands

**Define a command in a common crate**
```rust
// common crate: cmd.rs

#[tauri_interop::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
```

**And just use it in the UI**
```rust
// ui crate

async fn call_greet(name: &str) -> String {
    common::cmd::greet(&name).await
}
```


### Events (requires the `event` feature)

**Define a struct with some field**
```rust
// common crate: model.rs

// Event will generate new items which can then be used to emit and listen to values
#[derive(Default, Event)]
pub struct FooBar { // mod foo_bar
    foo: String, // struct FFoo
    pub bar: bool, // struct FBar
}
```

**Define a command which can use the instance when registered to tauri's state system**
```rust
// common crate: cmd.rs
#[tauri_interop::command]
pub fn emit_bar(state: TauriState<FooBar>, handle: TauriAppHandle) {
    state.update::<foo_bar::FBar>(&handle, true).unwrap();
}
```

**Listen to the field in the ui**
```rust
// ui crate

pub fn listen() {
    // default usage, returns a handle which needs to be hold in scope
    let listen_handle = FooBar::listen_to::<foo_bar::FBar>(|echo| log::info!("bar: {echo}"));

    // with feature: leptos, integrates nicely into the component system without needing to worry about the handle
    let signal = FooBar::use_field::<foo_bar::FBar>(None);
}

```

## Compatability and requirements

- toolchain: `nightly`

| tauri-interop | tauri | leptos |
|---------------|-------|--------|
| <= 2.1.6      | 1.6   | 0.6    |
| \>= 2.2.0     | 2     | 0.7    |

## Getting started

There are two concepts that generally make sense when using a single language for the entire codebase. Either separating
the host crate into a library and binary, where the library will build a common-usage for the front and backend.
Or [creating a new crate](#creating-a-common-crate) that builds a common-usage for both sides.

With `tauri` v1 we could avoid creating a separate crate, but with v2 it seems to be necessary to separate the
common-usage into an own crate.

### Init a new project

```shell
cargo create-tauri-app --template <rust_based_framework> --yes <project_name>
```

> for the template a rust based framework like `leptos`, `yew`, `dioxus` or `sycamore` should be selected

**Toolchain:**

`tauri-interop` uses some features that require the use of the nightly toolchain. To enforce that the correct toolchain
is used in the project we add a `rust-toolchain.toml` file with the following content:

```toml
[toolchain]
channel = "nightly"
```

**Target specific inclusion:**

Because we will compile the common crate for both wasm and the host target we need to communicate to `cargo` that
certain crates should be compiled for both and others only for their corresponding targets. For example, `tauri-interop`
needs to be included for both host and wasm target, while `tauri` (host) and `serde-wasm-bindgen` (wasm) need to be only
included in one of each.

To separate these crates in the `Cargo.toml` we can use target specific dependency declaration. The following shows how
the separation will look.

```toml
[dependencies]
tauri-interop = { version = "*", features = [] }

# host target
[target.'cfg(not(target_family = "wasm"))'.dependencies]

# wasm target
[target.'cfg(target_family = "wasm")'.dependencies]

```

The same target specific inclusion can also be done in code with the following attributes:

```rust
// host target
#[cfg(not(target_family = "wasm"))]
struct HostTarget;

// wasm target
#[cfg(target_family = "wasm")]
struct WasmTarget;
```

### Creating a common crate

Create a new crate (we will refer to it as `common` later on) and add it as a member to the workspace (see root
`Cargo.toml` in the `workspace` section). After the crate is a workspace member we can add `tauri-interop` to it and
separate the dependency section like previously mentioned.
section so that we don't get any compilation errors caused because of missing crates.

When everything is done, it should be possible to add the new crate to the wasm and host crate. Which finishes the
initial setup.

### Usage of the new structure

Now that we have a unified place where we can place common code, we can use the strong advantage of writing the commands
only once, instead of writing additional commands for our ui code.

As an example, we can move the templates `greet` command defined in `src-tauri/src/lib.rs` into our new `common` crate. 
For that we need to add a new module/file, which we will name `cmd.rs`. The new module is necessary because
due to a restriction on how the commands by `tauri` work (see the second notice in the [Basic Example](https://tauri.app/develop/calling-rust/#basic-example)). 
Regardless the restrictions we need to adjust the `greet` command slightly by making it public (so we can access it 
later from our ui code) and replacing `tauri::command` with `tauri_interop::command` (so that the command can be also
called from our ui code).

Additionally, we can use `tauri_interop::collect_commands!()` to collect all commands in the current file and register 
them in our app with a newly generated `get_handlers` function. The `cmd.rs` should look something like this now: 

```rust
// cmd.rs

#[tauri_interop::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

tauri_interop::collect_commands!();
```

To use the `get_handlers` function we need to switch to where the `tauri::Builder` is constructed and register our command
handlers with the `invoke_handler`. By default, the `tauri::generate_handler!` macro is used to accomplish registering 
all commands, but that part is handled by `tauri-interop` no and can be easily accomplished by calling 
`common::cmd::get_handlers()` instead.

> To create more complex command constellations `tauri_interop::combine_handlers!()` is provided to merge commands
> defined in multiple modules.

```rust
// lib.rs

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(common::cmd::get_handlers())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Now we need to actually call our command in our `ui` code. We can simply do that by calling `common::cmd::greet`. 
Because we have a function that returns a value, the generated function is `async` and we need to await it.

```rust
async fn call_greet(name: &str) -> String {
    common::cmd::greet(&name).await
}
```

### Note

The library uses a resolver 2 features to allow easy inclusion without configuration. When working with virtual
workspaces the resolver defaults to 1. In that case it is required to set the resolver manually to version 2,  
otherwise
the [target specific compilation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies)
will not resolve correctly. When the wrong resolver is used, an error should state that the `Listen` trait is missing.

