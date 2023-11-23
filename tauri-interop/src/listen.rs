use js_sys::Function;
use serde::{Serialize, Deserialize};
use wasm_bindgen::{JsValue, closure::Closure};

pub type ListenResult = Result<ListenHandle, ListenError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload<T> {
    pub payload: T,
    event: String
}

#[derive(Debug, thiserror::Error)]
pub enum ListenError {
    #[error("The promise to register the listener failed: {0:?}")]
    PromiseFailed(JsValue),
    #[error("The function to detach the listener wasn't a function: {0:?}")]
    NotAFunction(JsValue)
}

pub struct ListenHandle {
    pub closure: Option<Closure<dyn Fn(JsValue)>>,
    detach_fn: Function
}

impl ListenHandle {
    pub fn new(closure: Closure<dyn Fn(JsValue)>, detach_fn: Function) -> Self {
        Self { closure: Some(closure), detach_fn }
    }

    pub fn detach_listen(self) {
        self.detach_fn.apply(&JsValue::null(), &js_sys::Array::new()).unwrap();
    }
}
