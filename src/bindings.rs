use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
    pub fn invoke(cmd: &str, args: JsValue);

    #[wasm_bindgen(js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
    pub async fn async_invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(catch, js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
    pub async fn invoke_catch(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[cfg(feature = "listen")]
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    pub fn listen(event: &str, closure: &Closure<dyn Fn(JsValue)>) -> js_sys::Promise;
}