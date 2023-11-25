# Tauri-Interop

What this crate tries to achieve:
- generate a wasm-function from your defined `tauri::command` by using `tauri_interop::command` instead
- collect all defined `tauri_interop::command` by invoking `tauri_interop::collect_commands!()` at the end of a file
- a simple way to emit and listen to a state change from the backend using (requires the `listen` feature)


## Basic usage:

### Command (Frontend => Backend Communication)
Definition for both tauri supported triplet and wasm:
```
#[tauri_interop::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
```
let greetings = greet("frontend").await;
```

### Event (Backend => Frontend Communication)
Definition for both tauri supported triplet and wasm:
```
#[tauri_interop::emit_or_listen]
pub struct Test {
    pub foo: String,
    pub bar: i32,
}
```

Using `tauri_interop::emit_or_listen` does provides the command with two macros,
which are used depending on the `target_family`
  - `tauri_interop::listen_to` is used when compiling to `wasm`
  - `tauri_interop::emit` is used otherwise

To emit a variable from the above struct (which is mostly intended to be used as state) in the host triplet
```
let test = Test {
    foo: "foo".into(),
    bar: 69
};

// where `handle` is `tauri::AppHandle`
test.emit(&handle, TestEmit::Foo);
```

the above emitted value can then be received in wasm as:
```
// Test::listen_to_<field>
let listen_handle = Test::listen_to_foo(|foo| /* use received foo here */)
```

## Known Issues:
- arguments used in `tauri::command` beginning with `_` aren't supported yet
  - due to [tauri internally converting the argument name](https://tauri.app/v1/guides/features/command#passing-arguments), 
    which results in losing the _ at the beginning
