# Tauri-Interop

[![Latest version](https://img.shields.io/crates/v/tauri-interop.svg)](https://crates.io/crates/tauri-interop)
[![Documentation](https://docs.rs/tauri-interop/badge.svg)](https://docs.rs/tauri-interop)
![License](https://img.shields.io/crates/l/tauri-interop.svg)

This crate tries to provide a general more enjoyable experience for developing tauri apps with a rust frontend.
> tbf it is a saner approach to write the app in a mix of js + rust, because the frameworks are more mature, there are
> way more devs who have experience with js and their respective frameworks etc...
> 
> but tbh... because something is saner, doesn't stop us from doing things differently ^ヮ^

Writing an app in a single language give the option to build a common crate/module which connects the backend and 
frontend. A common model itself can be most of the time easily compile to both architectures (arch's) when the types 
are compatible with both. The commands on the other hand don't have an option to be compiled to wasm. Which means they
need to be handled manually or be call via a wrapper/helper each time. 

Repeating the implementation and handling for a function that is already defined properly seems to be a waste of time.
For that reason this crate provides the `tauri_interop::command` macro. This macro is explained in detail in the 
[command representation](#command-representation-hostwasm) section. This new macro provides the option to invoke the 
command in wasm and by that call the defined command in tauri. On the other side, when compiling for tauri additionally 
to the tauri logic, the macro provides the option to collect all commands in a single file via the invocation of the 
`tauri_interop::collect_commands` macro at the end of the file (see [command](#command-frontend--backend-communication)).

In addition, some quality-of-life macros are provided to eas some inconveniences when compiling to multiple arch's. See
the [QOL](#qol-macros) section.

**Feature `event`**:

Tauri has an [event](https://tauri.app/v1/guides/features/events) mechanic with that the tauri side can communicate to
the frontend. The usage is not as intuitive and has to some inconveniences that make it quite hard to recommend. To 
improve the usage the crate provides the derive-marcos `Event`, `Emit` and `Listen`. The `Event` macro is just a 
conditional wrapper that expands to `Emit` for the tauri compilation and `Listen` for the wasm compilation. It is 
the indent way to use this feature. The usage is explained in more detail in the [event](#event-backend--frontend-communication) 
section.

## Basic usage:

> **Disclaimer**:
>
> Some examples in this documentation can't be executed with doctests due to
> required wasm target and tauri modified environment (see [withGlobalTauri](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri))

### Command (Frontend => Backend Communication)
> For more examples see [cmd.rs](./test-project/api/src/cmd.rs) in test-project

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
- it adds an entry to `tauri_interop::collect_commands!()` so that the generated `get_handlers()` function includes 
  the given commands for the tauri context
  - it can be invoked inside `.invoke_handler` methode of the `tauri::Builder` instead of the usual `generate_handler` macro
  - it is not generated when targeting `wasm`
  

The defined command above can then be used in wasm as below. Due to receiving data from 
tauri via a promise, the command response has to be awaited.
```rust , ignore
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

#### Command representation Host/Wasm

- the returned type of the wasm binding should be 1:1 the same type as send from the "backend" 
  - technically all commands need to be of type `Result<T, E>` because there is always the possibility of a command 
    getting called, that isn't registered in the context of tauri
    - when using `tauri_interop::collect_commands!()` this possibility is fully™️ removed
    - for convenience, we ignore that possibility, and even if the error occurs it will be logged into the console
- all arguments with `tauri` in their name (case-insensitive) are removed as argument in a defined command
  - that includes `tauri::*` usages and `Tauri` named types
  - the crate itself provides type aliases for tauri types usable in a command (see [type_aliases](./src/command/type_aliases.rs))
- most return types are automatically determined
  - when using a return type with `Result` in the name, the function will also return a `Result`
  - that also means, if you create a type alias for `Result<T, E>` and don't include `Result` in the name of the alias, 
    it will not map the `Result` correctly

```rust , ignore-wasm32-unknown-unknown
// let _: () = trigger_something();
#[tauri_interop::command]
fn trigger_something(name: &str) {
    print!("triggers something, but doesn't need to wait for it")
}

// let value: String = wait_for_sync_execution("value").await;
#[tauri_interop::command]
fn wait_for_sync_execution(value: &str) -> String {
    format!("Has to wait that the backend completes the computation and returns the {value}")
}

// let result: Result<String, String> = asynchronous_execution(true).await;
#[tauri_interop::command]
async fn await_heavy_computing() { 
    std::thread::sleep(std::time::Duration::from_millis(5000))
}

// let result: Result<String, String> = asynchronous_execution(true).await;
#[tauri_interop::command]
async fn asynchronous_execution(change: bool) -> Result<String, String> {
    if change {
        Ok("asynchronous execution returning result, need Result in their type name".into())
    } else {
        Err("if they don't it, the error will be not be parsed/handled".into())
    }
}

// let _wait_for_completion: () = heavy_computation().await;
#[tauri_interop::command]
async fn heavy_computation() {
  std::thread::sleep(std::time::Duration::from_millis(5000))
}
```

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

### Event (Backend => Frontend Communication)
Definition for both tauri supported triplet and wasm:
```rust
use tauri_interop::Event;

#[derive(Default, Event)]
pub struct Test {
    foo: String,
    pub bar: bool,
}

// when main isn't defined, `super::Test` results in an error
fn main() {}
```

When using the derive macro `tauri_interop::Event` it expands depending on the `target_family` to
  - derive trait `tauri_interop::Listen` (when compiling to `wasm`)
  - derive trait `tauri_interop::Emit` (otherwise)

To emit a variable from the above struct (which is mostly intended to be used as state) in the host triplet
```rust , ignore-wasm32-unknown-unknown
use tauri_interop::{Event, event::Emit};

#[derive(Default, Event)]
pub struct Test {
    foo: String,
    pub bar: bool,
}

// via `tauri_interop::Emit` a new module named after the struct (as snake_case) 
// is created where the struct Test is defined, here it creates module `test`
// in this module the related Fields are generated

// one context where `tauri::AppHandle` can be obtained
#[tauri_interop::command]
fn emit_bar(handle: tauri::AppHandle) {
    let mut t = Test::default();

    t.emit::<test::Foo>(&handle); // emits the current state: `false`
}

// a different context where `tauri::AppHandle` can be obtained
fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let handle: tauri::AppHandle = app.handle();
      
      let mut t = Test::default();

      // to emit and update a field an update function for each field is generated
      t.update::<test::Foo>(&handle, "Bar".into()); // assigns "Bar" to t.foo and emits the same value

      Ok(())
    });
}
```

the above emitted value can then be received in wasm as:
```rust , ignore
use tauri_interop::Event;

#[derive(Default, Event)]
pub struct Test {
    foo: String,
    pub bar: bool,
}

async fn main() {
  use tauri_interop::event::listen::Listen;

  let _listen_handle: ListenHandle<'_> = Test::listen_to::<test::Foo>(|foo| { /* use received foo: String here */ }).await;
}
```

The `ListenHandle` contains the provided closure and the "unlisten" method. It has to be hold in scope as long 
as the event should be received. Dropping it will automatically detach the closure from the event. See 
[cmd.rs](./test-project/api/src/cmd.rs) for other example how it could be used.

#### Feature: leptos
When the `leptos` feature is enabled the `use_field` method is added to the `Listen` trait when compiling to wasm. 
The method takes care of the initial asynchronous call to register the listener and will hold the handle in scope 
as long as the leptos component is rendered.

```rust , ignore
use tauri_interop::Event;

#[derive(Default, Event)]
pub struct Test {
    foo: String,
    pub bar: bool,
}

fn main() {
  use tauri_interop::event::listen::Listen;

  let foo: leptos::ReadSignal<String> = Test::use_field::<test::Foo>(String::default());
}
```
