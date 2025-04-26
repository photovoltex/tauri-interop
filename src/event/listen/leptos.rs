use super::{Field, Listen, ListenHandle, Parent};
use leptos::prelude::*;

pub type ListenSignal<P: Parent, F: Field<P>> = ReadSignal<<F as Field<P>>::Type, LocalStorage>;

impl ListenHandle {
    /// Registers a given event and binds a returned signal to these event changes
    ///
    /// Providing [None] will unwrap into the default value. When feature `initial_value`
    /// is enabled [None] will try to get the value from tauri.
    ///
    /// Internally it stores a created [ListenHandle] for `event` in a [RwSignal] to hold it in
    /// scope, while it is used in a leptos [component](https://docs.rs/leptos_macro/0.5.2/leptos_macro/attr.component.html)
    pub fn use_register<P, F: Field<P>>(
        initial_value: Option<F::Type>,
    ) -> ReadSignal<<F as Field<P>>::Type, LocalStorage>
    where
        P: Parent,
    {
        #[cfg(any(all(target_family = "wasm", feature = "initial_value")))]
        let acquire_initial_value = initial_value.is_none();
        let (signal, set_signal) = signal_local(initial_value.unwrap_or_default());

        // creating this signal in a leptos component holds the value in scope, and drops it automatically
        let handle = RwSignal::new_local(None);
        leptos::task::spawn_local(async move {
            #[cfg(any(all(target_family = "wasm", feature = "initial_value")))]
            if acquire_initial_value {
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
