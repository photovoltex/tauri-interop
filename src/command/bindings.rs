use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// Fire and forget invoke/command call
    ///
    /// [Tauri Commands](https://tauri.app/v1/guides/features/command)
    #[wasm_bindgen(js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
    pub fn invoke(cmd: &str, args: JsValue);

    /// [invoke] variant that awaits the returned value
    ///
    /// [Async Commands](https://tauri.app/v1/guides/features/command/#async-commands)
    #[wasm_bindgen(js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
    pub async fn async_invoke(cmd: &str, args: JsValue) -> JsValue;

    /// [async_invoke] variant that additionally returns a possible error
    ///
    /// [Error Handling](https://tauri.app/v1/guides/features/command/#error-handling)
    #[wasm_bindgen(catch, js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
    pub async fn invoke_catch(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    /// The binding for the frontend that listens to events
    ///
    /// [Events](https://tauri.app/v1/guides/features/events)
    #[cfg(any(feature = "event", doc))]
    #[doc(cfg(feature = "event"))]
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"])]
    pub async fn listen(
        event: &str,
        closure: &Closure<dyn Fn(JsValue)>,
    ) -> Result<JsValue, JsValue>;
}

/// Wrapper for [async_invoke], to return an
/// expected [DeserializeOwned] object
pub async fn wrapped_async_invoke<T>(command: &str, args: JsValue) -> T
where
    T: DeserializeOwned,
{
    let value = async_invoke(command, args).await;
    serde_wasm_bindgen::from_value(value).expect("conversion error")
}

/// Wrapper for [invoke_catch], to return an
/// expected [Result<T, E>] where both generics are [DeserializeOwned]
pub async fn wrapped_invoke_catch<T, E>(command: &str, args: JsValue) -> Result<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    invoke_catch(command, args)
        .await
        .map(|value| serde_wasm_bindgen::from_value(value).expect("ok: conversion error"))
        .map_err(|value| serde_wasm_bindgen::from_value(value).expect("err: conversion error"))
}
