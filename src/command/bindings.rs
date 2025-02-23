use js_sys::{JsString, RegExp};
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// Binding for tauri's global invoke function
    ///
    /// - [Tauri Commands](https://v2.tauri.app/develop/calling-rust/)
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    /// The binding for the frontend that listens to events
    ///
    /// [Events](https://v2.tauri.app/develop/calling-frontend/)
    #[cfg(feature = "event")]
    #[doc(cfg(feature = "event"))]
    // for some reason this doc comment is seen as unused...
    #[allow(unused_doc_comments)]
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"])]
    pub async fn listen(
        event: &str,
        closure: &Closure<dyn Fn(JsValue)>,
    ) -> Result<JsValue, JsValue>;
}

enum InvokeResult {
    Ok(JsValue),
    Err(JsValue),
    NotRegistered,
}

/// Wrapper for [invoke], to handle an unregistered function
async fn wrapped_invoke(command: &str, args: JsValue) -> InvokeResult {
    match invoke(command, args).await {
        Ok(value) => InvokeResult::Ok(value),
        Err(value) => {
            if let Some(string) = value.dyn_ref::<JsString>() {
                let regex = RegExp::new("command (\\w+) not found", "g");
                if string.match_(&regex).is_some() {
                    log::error!("Error: {string}");
                    return InvokeResult::NotRegistered;
                }
            }

            InvokeResult::Err(value)
        }
    }
}

/// Wrapper for [wait_invoke], to send a command without waiting for it
pub fn fire_and_forget_invoke(command: &'static str, args: JsValue) {
    wasm_bindgen_futures::spawn_local(wait_invoke(command, args))
}

/// Wrapper for [invoke], to await a command execution without handling the returned values
pub async fn wait_invoke(command: &'static str, args: JsValue) {
    wrapped_invoke(command, args).await;
}

/// Wrapper for [invoke], to return an expected [DeserializeOwned] item
pub async fn return_invoke<T>(command: &str, args: JsValue) -> T
where
    T: Default + DeserializeOwned,
{
    match wrapped_invoke(command, args).await {
        InvokeResult::Ok(value) => serde_wasm_bindgen::from_value(value).unwrap_or_else(|why| {
            log::error!("Conversion failed: {why}");
            Default::default()
        }),
        _ => Default::default(),
    }
}

/// Wrapper for [invoke], to return an expected [Result<T, E>]
pub async fn catch_invoke<T, E>(command: &str, args: JsValue) -> Result<T, E>
where
    T: Default + DeserializeOwned,
    E: DeserializeOwned,
{
    match wrapped_invoke(command, args).await {
        InvokeResult::Ok(value) => {
            Ok(serde_wasm_bindgen::from_value(value).unwrap_or_else(|why| {
                log::error!("Conversion failed: {why}");
                Default::default()
            }))
        }
        InvokeResult::Err(value) => Err(serde_wasm_bindgen::from_value(value).unwrap()),
        InvokeResult::NotRegistered => Ok(Default::default()),
    }
}
