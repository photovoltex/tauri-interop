use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

pub type ListenResult = Result<ListenHandle, ListenError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload<T> {
    pub payload: T,
    event: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ListenError {
    #[error("The promise to register the listener failed: {0:?}")]
    PromiseFailed(JsValue),
    #[error("The function to detach the listener wasn't a function: {0:?}")]
    NotAFunction(JsValue),
}

pub struct ListenHandle {
    pub closure: Option<Closure<dyn Fn(JsValue)>>,
    detach_fn: Function,
}

impl ListenHandle {
    pub fn new(closure: Closure<dyn Fn(JsValue)>, detach_fn: Function) -> Self {
        Self {
            closure: Some(closure),
            detach_fn,
        }
    }

    pub fn detach_listen(self) {
        self.detach_fn
            .apply(&JsValue::null(), &js_sys::Array::new())
            .unwrap();
    }
}

pub async fn listen<T>(event: &str, callback: impl Fn(T) + 'static) -> ListenResult
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let closure = wasm_bindgen::prelude::Closure::new(move |value| {
        let payload: Payload<T> = serde_wasm_bindgen::from_value(value).expect("serializable");

        callback(payload.payload)
    });

    let ignore = crate::bindings::listen(event, &closure);
    let detach_fn = wasm_bindgen_futures::JsFuture::from(ignore)
        .await
        .map_err(ListenError::PromiseFailed)?
        .dyn_into::<Function>()
        .map_err(ListenError::NotAFunction)?;

    Ok(ListenHandle::new(closure, detach_fn))
}
