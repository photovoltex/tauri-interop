use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

/// The result type that is returned by [ListenHandle::register]
pub type ListenResult<'s> = Result<ListenHandle<'s>, ListenError>;

/// The generic payload received from [crate::bindings::listen] used for deserialization
#[derive(Debug, Serialize, Deserialize)]
pub struct Payload<T> {
    payload: T,
    event: String,
}

/// Errors that can occur during registering the callback in [ListenHandle::register]
#[derive(Debug, thiserror::Error)]
pub enum ListenError {
    /// The promised given by [crate::bindings::listen] failed to resolve
    #[error("The promise to register the listener failed: {0:?}")]
    PromiseFailed(JsValue),
    /// The returned value from the resolved [js_sys::Promise] retrieved
    /// from [crate::bindings::listen] wasn't a function
    #[error("The function to detach the listener wasn't a function: {0:?}")]
    NotAFunction(JsValue),
}

/// Handle which holds the unlisten function and the correlated callback
pub struct ListenHandle<'s> {
    /// The callback which is invoke for the registered event
    pub closure: Option<Closure<dyn Fn(JsValue)>>,
    event: &'s str,
    detach_fn: Function,
}

impl<'s> ListenHandle<'s> {
    /// Registers a given event with the correlation callback and returns a [ListenResult]
    pub async fn register<T>(event: &str, callback: impl Fn(T) + 'static) -> ListenResult
    where
        T: for<'de> Deserialize<'de> + Serialize,
    {
        let closure = wasm_bindgen::prelude::Closure::new(move |value| {
            let payload: Payload<T> = serde_wasm_bindgen::from_value(value)
                .map_err(|why| log::error!("{why:?}"))
                .expect("passed value from backend didn't serialized correctly");

            callback(payload.payload)
        });

        let ignore = crate::bindings::listen(event, &closure);
        let detach_fn = wasm_bindgen_futures::JsFuture::from(ignore)
            .await
            .map_err(ListenError::PromiseFailed)?
            .dyn_into::<Function>()
            .map_err(ListenError::NotAFunction)?;
        let closure = Some(closure);

        Ok(ListenHandle {
            event,
            closure,
            detach_fn,
        })
    }

    /// Detaches the callback from the registered event
    pub fn detach_listen(self) {
        log::trace!("Detaching listener for {}", self.event);

        self.detach_fn
            .apply(&JsValue::null(), &js_sys::Array::new())
            .unwrap();
    }
}
