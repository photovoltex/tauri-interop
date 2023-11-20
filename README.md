# Tauri-Interop
> self is excluded by default (doesn't bring any benefits due to tauri::command not being able to be used in a impl/trait)

## TODO:
- should have a async and no async option (each with a catch option)

## Basic usage:
```rs
// provides required tauri wasm bindings (has to be called from root)
tauri_interop::init!();
```

### Commands
> notice: internally the arg name is converted to Camel (see 
> [Passing Arguments](https://tauri.app/v1/guides/features/command#passing-arguments) for more infos) \
> if at some point there is a panic because some arguments aren't provided, it is probably because of 
> something incorrect conversion internally, please report this as issue if u ever run into it

```rs
// if wasm, compile it with the interop crate
#[cfg_attr(target_family = "wasm", tauri_interop::invoke)]
// if native compile it with tauri
#[cfg_attr(not(target_family = "wasm"), tauri::command)]
pub fn greet(name: String) -> String {
    todo!()
}
```

### Backend to Frontend communication (feature: `listen`)
This bridge uses the global event mechanic exposed by tauri. ([docs](https://tauri.app/v1/guides/features/events/)) 
The basic concept is to create a "communication enum" (via the `tauri_interop::handle_emit_all`
proc_macro_attr), which is used to generated functions, which then can be used to communicate
between front and backend with lsp support. In the backend an `tauri::AppHandle` can only be
obtained in the `setup` or via a `tauri::command`. Because the idea is to have access to the
handle in any context, setting this up in the provided setup process is recommended. The general
idea is displayed below with a short example.

The tricky part of this whole process, is the handling in the frontend. Due to rust memory
management, we can't just create a closure, give it to a function and call it a day. No, no. With
rust we have to preserve these closures, so that they don't get free before they are ever used.
`tauri_interop::listen` generate a function for each enum variant. These generated function capture
the provided closure and registers them to the tauri event listener. But we are not safe yet,
because the registered closures are not context saved yet. The closures which need to be saved are
returned by each function. How they are saved is up to the user cause this depends on the app
structure (for example yew and leptos use very different approaches to state management). I 
provided a basic leptos example how to store these closures below (the provided context is 
probably not necessary for the example, but it hopefully gives a good idea what the whole point of 
it is).
> will get better examples (actual example projects) once i separate this crate from this project

#### Generation of bridge
```rs
#[cfg_attr(target_family = "wasm", tauri_interop::listen)]
#[cfg_attr(not(target_family = "wasm"), tauri_interop::handle_emit_all)]
#[derive(Debug)]
pub enum GreetEmit {
    Hello(String),
    User(User),
    // <EventName>(<Payload: Serialize + Clone>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User(String);
```

#### QOL: Generate Bridge from Struct
This example generates the above Enum which then generates the bridge as usual
```rs
#[tauri_interop::generate_emit_enum]
#[derive(Debug)]
pub struct Greet {
    hello: String,
    user: User,
    // <event_name>: <Payload: Serialize + Clone>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User(String);
```


#### Usage off generated bridge
- backend
```rs
#[derive(Debug)]
pub enum HandleCommand {
    Emit(Emit),
}

... {
    ...
    // the handle_tx can then be used from a different context to use the app handle
    let (handle_tx, mut handle_rx) = tokio::sync::mpsc::unbounded_channel::<HandleCommand>();

    tauri::Builder::default()
        .setup(|app| {
          let handle = app.handle();
          tauri::async_runtime::spawn(async move {
              loop {
                  let event = handle_rx.recv().await;
                  match event {
                      None => break,
                      Some(event) => match event {
                          HandleCommand::Emit(emit) => {
                              Emit::with_handle(emit, &handle).expect("no error")
                          }
                      },
                  }
              }
          });
        })
        ...
}
```

- frontend
```rs
use leptos::*;
use wasm_bindgen::{JsValue, prelude::Closure};

pub type StorableClosure = Closure<dyn Fn(JsValue)>;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (_, update_closure_store) = create_signal(cx, Vec::<StorableClosure>::new());
    provide_context(cx, update_closure_store);

    view! { cx,
      <UsesClosures />
    }
}

#[component]
pub fn UsesClosures(cx: Scope) -> impl IntoView {
    let store: WriteSignal<ClosureStore> = expect_context(cx);

    let closure_hello = tautops::player::listen_to_hello(move |string| {
        log::info!("hello {string}");
    });
    let closure_world = tautops::player::listen_to_world(move |user| {
        log::info!("received User: {user:?}");
    });

    store.update(|c| {
      c.push(closure_hello);
      c.push(closure_world);
    });

    view! { cx,
      <p> { "some view" } </p>
    }
}
```


## Advanced usage:
### Excluding internal data
> the function argument is removed via exact name matching (multiple matches, separated via comma, possible)

```rs
#[cfg_attr(target_family = "wasm", tauri_interop::invoke["state"])]
#[cfg_attr(not(target_family = "wasm"), tauri::command)]
pub fn internal_state_operation(name: String, state: State<InternalDataConstruct>) {
    todo!()
}
```

### Using Results
> currently catch_command is intended for using with Result<T, E> \
> this could change by adding more argument support further on to the command macro

```rs
#[cfg_attr(target_family = "wasm", tauri_interop::catch_invoke)]
#[cfg_attr(not(target_family = "wasm"), tauri::command)]
pub fn could_fail(name: String) -> Result<(), String> {
    todo!()
}
```