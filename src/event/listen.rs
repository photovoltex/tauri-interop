#[cfg(feature = "leptos")]
#[doc(cfg(feature = "leptos"))]
mod leptos;

use js_sys::Function;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::future::Future;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use crate::command::bindings::listen;

use super::Field;
#[cfg(doc)]
use super::{Emit, Parent};

/// The trait which needs to be implemented for a [Field]
///
/// Conditionally changes between [Listen] and [Emit]
#[cfg(target_family = "wasm")]
pub trait Parent = Listen;

/// The result type that is returned by [ListenHandle::register]
pub type ListenResult = Result<ListenHandle, ListenError>;

/// The generic payload received from [listen] used for deserialization
#[derive(Debug, Serialize, Deserialize)]
pub struct Payload<T> {
    payload: T,
    event: String,
}

/// Errors that can occur during registering the callback in [ListenHandle::register]
#[derive(Debug, thiserror::Error)]
pub enum ListenError {
    /// The promised given by [listen] failed to resolve
    #[error("The promise to register the listener failed: {0:?}")]
    PromiseFailed(JsValue),
    /// The returned value from the resolved [js_sys::Promise] retrieved
    /// from [listen] wasn't a function
    #[error("The function to detach the listener wasn't a function: {0:?}")]
    NotAFunction(JsValue),
}

/// Handle which holds the function to detach the listener and the correlated callback
pub struct ListenHandle {
    /// The callback which is invoked for the registered event
    ///
    /// The callback will get detached, when the handle is dropped. Alternatively it can
    /// also be given to the js runtime (see [Closure] `into_js_value`/`forget`). This isn't
    /// recommended because this will leak memory by default.
    pub closure: Option<Closure<dyn Fn(JsValue)>>,
    event: &'static str,
    detach_fn: Function,
}

impl Drop for ListenHandle {
    fn drop(&mut self) {
        self.detach_listen()
    }
}

impl ListenHandle {
    /// Registers a given event with the correlation callback and returns a [ListenResult]
    pub async fn register<T>(event: &'static str, callback: impl Fn(T) + 'static) -> ListenResult
    where
        T: DeserializeOwned,
    {
        let closure = Closure::new(move |value| {
            let payload: Payload<T> = serde_wasm_bindgen::from_value(value)
                .map_err(|why| log::error!("{why:?}"))
                .expect("passed value from backend didn't serialized correctly");

            callback(payload.payload)
        });

        let detach_fn = listen(event, &closure)
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
}

/// Trait that defines the available listen methods
pub trait Listen: Sized {
    /// Registers a callback to a [Field]
    ///
    /// Default Implementation: see [ListenHandle::register]
    ///
    /// ### Example
    ///
    /// ```ignore
    /// use tauri_interop::Event;
    ///
    /// #[derive(Default, Event)]
    /// pub struct Test {
    ///     foo: String,
    ///     pub bar: bool,
    /// }
    ///
    /// async fn main() {
    ///     use tauri_interop::event::listen::Listen;
    ///
    ///     let _listen_handle: ListenHandle<'_> = Test::listen_to::<test::Foo>(|foo| {
    ///         /* use received foo: String here */
    ///     }).await;
    /// }
    /// ```
    fn listen_to<F: Field<Self>>(
        callback: impl Fn(F::Type) + 'static,
    ) -> impl Future<Output = ListenResult>
    where
        Self: Parent,
    {
        ListenHandle::register(F::EVENT_NAME, callback)
    }

    /// Creates a signal to a [Field]
    ///
    /// Default Implementation: see [ListenHandle::use_register]
    ///
    /// ### Example
    ///
    /// ```ignore
    /// use tauri_interop::Event;
    ///
    /// #[derive(Default, Event)]
    /// pub struct Test {
    ///     foo: String,
    ///     pub bar: bool,
    /// }
    ///
    /// fn main() {
    ///   use tauri_interop::event::listen::Listen;
    ///
    ///   let foo: leptos::ReadSignal<String> = Test::use_field::<test::Foo>(String::default());
    /// }
    /// ```
    #[cfg(feature = "leptos")]
    #[doc(cfg(feature = "leptos"))]
    fn use_field<F: Field<Self>>(initial: Option<F::Type>) -> leptos::ListenSignal<Self, F>
    where
        Self: Parent,
    {
        ListenHandle::use_register::<Self, F>(initial)
    }
}
