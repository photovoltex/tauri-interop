use std::time::SystemTime;

use wasm_bindgen::closure::Closure;


#[::wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[::wasm_bindgen::prelude::wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    fn listen(event: &str, closure: &::wasm_bindgen::prelude::Closure<dyn Fn(::wasm_bindgen::JsValue)>) -> ::js_sys::Function;
}

fn main() {
    // if the project is loaded, currently rust-analyzer seems to be 
    // unable to compile different workspace-members with different targets, 
    // as that we get the "host" lsp suggestions and not the generated "wasm"
    // rust-analyzer.check.targets = [ "wasm32-unknown-unknown", "x86_64-unknown-linux-gnu" ] 
    // should fix that, but it doesn't seem to work as i wish it would?
    tauri_interop_api::cmd::empty_invoke();
    wasm_bindgen_futures::spawn_local(async {
        tauri_interop_api::cmd::invoke_promise_with_app_handle_as_argument().await.unwrap()
    });

    let closure = Closure::new(|_| println!("test"));
    let unlisten = listen("event", &closure);

    let now = SystemTime::now();

    while let Ok(elapsed) = now.elapsed() {
        if elapsed.as_millis() < 5000 {
            break;
        }
    }

    let _ = unlisten.call0(&Default::default());

    let now = SystemTime::now();
    while let Ok(elapsed) = now.elapsed() {
        if elapsed.as_millis() < 5000 {
            break;
        }
    }
}
