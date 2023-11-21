use js_sys::Function;
use wasm_bindgen::{JsValue, closure::Closure};

#[derive(Debug)]
pub enum ListenError {
    PromiseFailed,
    NotAFunction
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
