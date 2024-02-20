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
- it adds an entry to `tauri_interop::collect_commands!()` so that the generated 
  `get_commands()` function includes/registers the given commands for the tauri context

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

- technically all commands need to be of type `Result<T, E>`
  - where E is a error that the command isn't defined, if not explicitly redefined via the return type
  - this error will show up in the web console, so there shouldn't be a problem ignoring it
- all arguments with type "State", "AppHandle" and "Window" are removed automatically
> the current implementation relies on the name of the type and can not separate between a 
> tauri::State and a self defined "State" struct
- asynchronous commands are values as is seen [async-commands](https://tauri.app/v1/guides/features/command#async-commands) for a detail explanation

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
async fn asynchronous_execution(change: bool) -> Result<String, String> {
    if change {
        Ok("asynchronous execution requires result definition".into())
    } else {
        Err("and ".into())
    }
}

// let _wait_for_completion: () = asynchronous_execution(true).await;
#[tauri_interop::command]
async fn heavy_computation() {
  std::thread::sleep(std::time::Duration::from_millis(5000))
}
```

### Event (Backend => Frontend Communication)
Definition for both tauri supported triplet and wasm:
```rust
#[derive(Default)]
#[tauri_interop::emit_or_listen]
pub struct Test {
    foo: String,
    pub bar: bool,
}

// when main isn't defined, `super::Test` results in an error
fn main() {}
```

Using `tauri_interop::emit_or_listen` does provides the command with two macros,
which are used depending on the `target_family`
  - `tauri_interop::listen_to` is used when compiling to `wasm`
  - derive trait `tauri_interop::Emit` is used otherwise

To emit a variable from the above struct (which is mostly intended to be used as state) in the host triplet
```rust , ignore-wasm32-unknown-unknown
#[derive(Default)]
#[tauri_interop::emit_or_listen]
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
#[tauri_interop::emit_or_listen]
pub struct Test {
    foo: String,
    pub bar: bool,
}

async fn main() {
  use tauri_interop::event::listen::Listen;

  let _listen_handle: ListenHandle<'_> = Test::listen_to::<test::Foo>(|foo| { /* use received foo: String here */ }).await;
}
```

The `liste_handle` contains the provided closure and the "unlisten" method. It has to be hold in scope as long 
as the event should be received. Dropping it will automatically detach the closure from the event. See 
[cmd.rs](./test-project/api/src/cmd.rs) for other example how it could be used.

#### Feature: leptos
When the `leptos` feature is enabled it will add additional `use_<field>` methods on the provided struct.
These methods take care of the required initial asynchron call to register the listener and will hold the
handle in scope as long as the component is rendered.

```rust , ignore
#[tauri_interop::emit_or_listen]
pub struct Test {
    foo: String,
    pub bar: bool,
}

fn main() {
  use tauri_interop::event::listen::Listen;

  let (foo: leptos::ReadSignal<String>, set_foo: leptos::WriteSignal<String>) = Test::use_field::<test::Foo>(String::default());
}
```

## Known Issues:
- arguments used in `tauri::command` beginning with `_` aren't supported yet
  - due to [tauri internally converting the argument name](https://tauri.app/v1/guides/features/command#passing-arguments), 
    which results in losing the _ at the beginning
- feature: leptos
  - sometimes a closure is accessed after being dropped
  - that is probably a race condition where the unlisten function doesn't detach the callback fast enough
