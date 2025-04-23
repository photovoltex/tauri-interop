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

- generate a wasm function out of the defined tauri-command
- collect and register all defined tauri-commands
- QOL-macros to exclude multiple imports in wasm or the host architecture
- easier usage of [tauri's event feature](https://tauri.app/v1/guides/features/events/)

## Compatability and requirements

- toolchain: `nightly`

| tauri-interop | tauri | leptos |
|---------------|-------|--------|
| <= 2.1.6      | 1.6   | 0.6    |
| current       | 2     | 0.7    |

### Required crates

|                      | host | wasm |
|----------------------|------|------|
| `log`                | x    | x    |
| `tauri`              | x    | -    |
| `serde`              | (x)  | x    |
| `serde-wasm-bindgen` | -    | x    |
| `leptos`             | -    | x1   |

- x: required
- x1: only required when the `leptos` feature is in use
- -: not compatible
- (x): possible, but not required

Reasons for inclusion:

- `log`: provides logging what `tauri-interop` registers, sends or emits
- `tauri`: to wrap the tauri macro and integrate with tauri
- `serde`/`serde-wasm-bindgen`: de-/serialization for objects received and sent via the ipc between ui and host
- `leptos`: for some specific leptos integration (only required when the corresponding feature is enabled)

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
log = "0.4"

# host target
[target.'cfg(not(target_family = "wasm"))'.dependencies]
tauri = { version = "2", features = [] }

# wasm target
[target.'cfg(target_family = "wasm")'.dependencies]
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
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
separate the dependency section like previously mentioned. Additionally add the crates mentioned in
the [required crates](#required-crates)
section so that we don't get any compilation errors caused because of missing crates.

When everything is done, it should be possible to add the new crate to the wasm and host crate. Which finishes the
initial setup.

### Usage of the new structure

Now that we have a unified place where we can place common code, we can use the strong advantage of writing the wasm
and host part in one language by only writing our commands once.

For that we can move the templates `greet` command defined in `src-tauri/src/lib.rs` into our new `common` crate. For
that we need to add a new module (will be referred as `cmd` or `cmd.rs` later). This is necessary because of a
restriction how the collection of commands and the command definition works from `tauri`. The important detail is, that
as long as the commands are not defined in `lib.rs` the restriction are neglectable. Regardless the restrictions we need
to modify the `greet` command slightly by making it public and replacing `tauri::command` with `tauri_interop::command`.
`tauri_interop::command` is a wrapper for `tauri::command` when compiling to the host target, but will generate wasm
bindings when compiling to wasm.

With that done, our main binary can't find the `greet` command anymore to include in the `tauri::generate_handler!`
macro. To resolve this we need to call the `tauri_interop::collect_commands!()` macro at the end of the file, where we
moved the `greet` command, which should be `cmd.rs`.

The call of `tauri_interop::collect_commands!()` will then generate a function called `get_handlers` in the module
where it was called. This function is intended to be called in place of `tauri::generate_handler!` and will add the
commands automatically annotated with `tauri_interop::command` to our tauri app. To use it in our example we now need to
replace `tauri::generate_handler![greet]` with `common::cmd::get_handlers()` (requires `cmd` to be public).

> To create more complex command constellations `tauri_interop::combine_handlers!()` is provided to merge commands
> defined in multiple modules.

Now we need to actually call our newly defined command in from our `common` crate in our `wasm` code. To do that we need
to go to `src/app.rs` and replace `invoke("greet", args).await.as_string().unwrap()` with
`common::cmd::greet(&name).await`.

> By doing that, we can also remove all the overhead (`invoke` and `GreetArgs`) which is usually necessary to write by
> ourselves, but is now automatically generated by `tauri-interop` in addition to providing correct types for both the
> parameters and return type.

### Note

The library uses a resolver 2 features to allow easy inclusion without configuration. When working with virtual
workspaces the resolver defaults to 1. In that case it is required to set the resolver manually to version 2,  
otherwise
the [target specific compilation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies)
will not resolve correctly. When the wrong resolver is used, an error should state that the `Listen` trait is missing.

