use serde::de::DeserializeOwned;
use wasm_bindgen::JsValue;

/// Wrapper for [crate::bindings::async_invoke], to return an
/// expected [DeserializeOwned] object
pub async fn async_invoke<T>(command: &str, args: JsValue) -> T
where
    T: DeserializeOwned,
{
    let value = crate::bindings::async_invoke(command, args).await;
    serde_wasm_bindgen::from_value(value).expect("conversion error")
}

/// Wrapper for [crate::bindings::invoke_catch], to return an
/// expected [Result<T, E>] where [T] and [E] is [DeserializeOwned]
pub async fn invoke_catch<T, E>(command: &str, args: JsValue) -> Result<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    crate::bindings::invoke_catch(command, args)
        .await
        .map(|value| serde_wasm_bindgen::from_value(value).expect("ok: conversion error"))
        .map_err(|value| serde_wasm_bindgen::from_value(value).expect("err: conversion error"))
}
