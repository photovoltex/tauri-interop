# Tauri-Interop

[![Latest version](https://img.shields.io/crates/v/tauri-interop.svg)](https://crates.io/crates/tauri-interop)
[![Documentation](https://docs.rs/tauri-interop/badge.svg)](https://docs.rs/tauri-interop)
![License](https://img.shields.io/crates/l/tauri-interop.svg)

What this crate tries to achieve:
- generate a equal wasm-function from your defined `tauri::command`
- collect all defined `tauri::command`s without adding them manually
- a simplified way to sending events from tauri and receiving them in the frontend


## Basic usage:


> **Disclaimer**:
>
> Some examples in this documentation can't be executed with doctests due to
> required wasm target and tauri modified environment (see [withGlobalTauri](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri))

### Command (Frontend => Backend Communication)
Definition for both tauri supported triplet and wasm:
```rust , ignore-wasm32-unknown-unknown
#[tauri_interop::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// generated the `get_handlers()` function
tauri_interop::collect_commands!();

fn main() {
  tauri::Builder::default()
    // This is where you pass in the generated handler collector
    .invoke_handler(get_handlers());
}
```

Using `tauri_interop::command` does two things:
- it provides the command with two macros which are used depending on the `target_family`
  - `tauri_interop::binding` is used when compiling to `wasm`
  - `tauri::command` is used otherwise
- it adds an entry to `tauri_interop::collect_commands!()` so that the generated 
  `get_commands()` function includes/registers the given commands for the tauri context

The defined command above can then be used in wasm as below. Due to receiving data from 
tauri via a promise, the command response has to be awaited.
```rust , ignore
// for testing this code is ignore due to required wasm target and tauri environment
#[tauri_interop::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    console_log::init_with_level(log::Level::Info).unwrap();

    wasm_bindgen_futures::spawn_local(async move { 
        let greetings = greet("frontend").await;
        log::info!("{greetings}");
    });
}
```

### Event (Backend => Frontend Communication)
Definition for both tauri supported triplet and wasm:
```rust , ignore-wasm32-unknown-unknown
#[derive(Default)]
#[tauri_interop::emit_or_listen]
pub struct Test {
    foo: String,
    pub bar: bool,
}
```

Using `tauri_interop::emit_or_listen` does provides the command with two macros,
which are used depending on the `target_family`
  - `tauri_interop::listen_to` is used when compiling to `wasm`
  - `tauri_interop::emit` is used otherwise

To emit a variable from the above struct (which is mostly intended to be used as state) in the host triplet
```rust , ignore-wasm32-unknown-unknown
#[derive(Default)]
#[tauri_interop::emit_or_listen]
pub struct Test {
    foo: String,
    pub bar: bool,
}

// one context where `tauri::AppHandle` can be obtained
#[tauri_interop::command]
fn emit_bar(handle: tauri::AppHandle) {
    let mut test = Test::default();

    test.emit(&handle, TestEmit::Bar); // emits `false`
    test.bar = true;
    test.emit(&handle, TestEmit::Bar); // emits updated value `true`
}

// a different context where `tauri::AppHandle` can be obtained
fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let handle: tauri::AppHandle = app.handle();
      
      let mut test = Test::default();

      // to emit and update an field an update function for each field is generated
      test.update_foo(&handle, "Bar".into()); // emits '"Bar"'

      Ok(())
    });
}
```

the above emitted value can then be received in wasm as:
```rust , ignore
// this code is ignore due to required target wasm
#[tauri_interop::emit_or_listen]
pub struct Test {
    foo: String,
    pub bar: bool,
}

let listen_handle = Test::listen_to_foo(|foo| { /* use received foo here */ })
```

## Known Issues:
- arguments used in `tauri::command` beginning with `_` aren't supported yet
  - due to [tauri internally converting the argument name](https://tauri.app/v1/guides/features/command#passing-arguments), 
    which results in losing the _ at the beginning
