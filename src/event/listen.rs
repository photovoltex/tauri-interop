use js_sys::Function;
#[cfg(feature = "leptos")]
use leptos::ReadSignal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use crate::command::bindings::listen;

#[cfg(doc)]
use super::{Emit, Parent};
use super::Field;

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

    /// Registers a given event and binds a returned signal to these event changes
    ///
    /// Providing [None] will unwrap into the default value. When feature `initial_value`
    /// is enabled [None] will try to get the value from tauri.
    ///
    /// Internally it stores a created [ListenHandle] for `event` in a [leptos::RwSignal] to hold it in
    /// scope, while it is used in a leptos [component](https://docs.rs/leptos_macro/0.5.2/leptos_macro/attr.component.html)
    #[cfg(feature = "leptos")]
    #[doc(cfg(feature = "leptos"))]
    pub fn use_register<P, F: Field<P>>(initial_value: Option<F::Type>) -> ReadSignal<F::Type>
    where
        P: Parent,
    {
        use leptos::SignalSet;

        let acquire_initial_value = initial_value.is_none();
        let (signal, set_signal) = leptos::create_signal(initial_value.unwrap_or_default());

        // creating this signal in a leptos component holds the value in scope, and drops it automatically
        let handle = leptos::create_rw_signal(None);
        leptos::spawn_local(async move {
            if cfg!(feature = "initial_value") && acquire_initial_value {
                match F::get_value().await {
                    Ok(value) => set_signal.set(value),
                    Err(why) => log::error!("{why}"),
                }
            }

            let listen_handle = ListenHandle::register(F::EVENT_NAME, move |value: F::Type| {
                log::trace!("update for {}", F::EVENT_NAME);
                set_signal.set(value)
            })
            .await
            .unwrap();

            // it could be that the component doesn't live long enough, so we just try to set it
            handle.try_set(Some(listen_handle));
        });

        signal
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
    ) -> impl std::future::Future<Output = ListenResult>
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
    fn use_field<F: Field<Self>>(initial: Option<F::Type>) -> ReadSignal<F::Type>
    where
        Self: Parent,
    {
        ListenHandle::use_register::<Self, F>(initial)
    }
}
