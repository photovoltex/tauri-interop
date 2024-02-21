use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

#[cfg(feature = "leptos")]
use leptos::{ReadSignal, WriteSignal};

use super::Field;

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
    ///
    /// The callback will get detached, when the handle is dropped. Alternatively it can
    /// also be given to the js runtime (see [Closure] `into_js_value`/`forget`). This isn't
    /// recommended because this will leak memory by default.
    pub closure: Option<Closure<dyn Fn(JsValue)>>,
    event: &'s str,
    detach_fn: Function,
}

impl Drop for ListenHandle<'_> {
    fn drop(&mut self) {
        self.detach_listen()
    }
}

impl<'s> ListenHandle<'s> {
    /// Registers a given event with the correlation callback and returns a [ListenResult]
    pub async fn register<T>(event: &str, callback: impl Fn(T) + 'static) -> ListenResult
    where
        T: for<'de> Deserialize<'de>,
    {
        let closure = wasm_bindgen::prelude::Closure::new(move |value| {
            let payload: Payload<T> = serde_wasm_bindgen::from_value(value)
                .map_err(|why| log::error!("{why:?}"))
                .expect("passed value from backend didn't serialized correctly");

            callback(payload.payload)
        });

        let detach_fn = crate::bindings::listen(event, &closure)
            .await
            .map_err(ListenError::PromiseFailed)?
            .dyn_into()
            .map_err(ListenError::NotAFunction)?;
        let closure = Some(closure);

        Ok(ListenHandle {
            event,
            closure,
            detach_fn,
        })
    }

    /// Detaches the callback from the registered event
    pub fn detach_listen(&mut self) {
        log::trace!("Detaching listener for {}", self.event);

        self.detach_fn
            .apply(&JsValue::null(), &js_sys::Array::new())
            .unwrap();
    }

    /// Registers a given event and binds a returned signal to these event changes
    ///
    /// Internally it stores a created [ListenHandle] for `event` in a [leptos::RwSignal] to hold it
    /// scope, while it is used in a [leptos::component](https://docs.rs/leptos_macro/0.5.2/leptos_macro/attr.component.html)
    #[cfg(feature = "leptos")]
    pub fn use_register<T>(event: &'static str, initial_value: T) -> (ReadSignal<T>, WriteSignal<T>)
    where
        T: for<'de> Deserialize<'de>,
    {
        use leptos::SignalSet;

        let (signal, set_signal) = leptos::create_signal(initial_value);

        // creating this signal in a leptos::component holdes the value in scope, and drops it automatically
        let handle = leptos::create_rw_signal(None);
        leptos::spawn_local(async move {
            let listen_handle = ListenHandle::register(event, move |value: T| {
                log::trace!("update for {}", event);
                set_signal.set(value);
            })
            .await
            .unwrap();

            // it could be that the component doesn't live long enough, so we just try to set it
            handle.try_set(Some(listen_handle));
        });

        (signal, set_signal)
    }
}

/// Trait that defines the available listen methods
pub trait Listen {
    /// Registers an callback to a [Field]
    ///
    /// Default Implementation: see [ListenHandle::register]
    fn listen_to<'r, F: Field<Self>>(
        callback: impl Fn(F::Type) + 'static,
    ) -> impl std::future::Future<Output = ListenResult<'r>>
    where
        Self: Sized + super::Parent,
    {
        ListenHandle::register(F::EVENT_NAME, callback)
    }

    /// Creates a signal to a [Field]
    ///
    /// Default Implementation: see [ListenHandle::use_register]
    #[cfg(feature = "leptos")]
    fn use_field<F: Field<Self>>(initial: F::Type) -> (ReadSignal<F::Type>, WriteSignal<F::Type>)
    where
        Self: Sized + super::Parent,
    {
        ListenHandle::use_register(F::EVENT_NAME, initial)
    }
}
