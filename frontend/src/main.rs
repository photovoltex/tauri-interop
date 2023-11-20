fn main() {
    // if the project is loaded, currently rust-analyzer seems to be 
    // unable to compile different workspace-members with different targets, 
    // as that we get the "host" lsp suggestions and not the generated "wasm"
    // rust-analyzer.check.targets = [ "wasm32-unknown-unknown", "x86_64-unknown-linux-gnu" ] 
    // should fix that, but it doesn't seem to work as i wish it would?
    tauri_interop_api::cmd::empty_invoke();
    wasm_bindgen_futures::spawn_local(async {
        tauri_interop_api::cmd::invoke_promise_with_app_handle_as_argument().await.unwrap()
    })
}
