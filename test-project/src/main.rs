#![allow(clippy::disallowed_names)]

use api::event::listen::Listen;
use api::model::{test_mod, TestState};
use gloo_timers::callback::Timeout;
#[cfg(feature = "leptos")]
use leptos::{component, create_signal, view, IntoView};

fn main() {
    console_log::init_with_level(log::Level::Trace).expect("no errors during logger init");
    console_error_panic_hook::set_once();

    api::command::empty_invoke();
    api::command::underscore_invoke(69);

    wasm_bindgen_futures::spawn_local(async {
        log::info!("{}", api::command::greet("frontend").await);

        api::command::await_heavy_computing().await;
        log::info!("heavy computing finished")
    });

    wasm_bindgen_futures::spawn_local(async move {
        let handle_bar = TestState::listen_to::<test_mod::Bar>(|echo| log::info!("bar: {echo}"))
            .await
            .unwrap();

        // with the move here, we hold "handle" in scope... if we wouldn't do that
        // handle would be dropped already and we get errors that the closure isn't anymore in scope
        //
        // it can be fixed with `handle.closure.take().unwrap().forget()`
        // see the `Closure::forget` docs, why this isn't the recommended way
        Timeout::new(2000, move || drop(handle_bar)).forget();
    });

    Timeout::new(1000, api::command::emit).forget();
    Timeout::new(3000, api::command::emit).forget();

    #[cfg(feature = "leptos")]
    Timeout::new(5000, || leptos::mount_to_body(|| view! { <App /> })).forget();
}

#[cfg(feature = "leptos")]
#[component]
fn App() -> impl IntoView {
    use leptos::{SignalGet, SignalSet};

    let (bar, set_bar) = create_signal(false);

    leptos::spawn_local(async move {
        let handle_bar = TestState::listen_to::<test_mod::Bar>(move |bar| set_bar.set(bar))
            .await
            .unwrap();

        Timeout::new(5000, move || drop(handle_bar)).forget();
    });

    view! {
        <div>
            <Foo/>
            {move || if bar.get() {
                "No Foo".into_view()
            } else {
                Foo.into_view()
            }}
        </div>
    }
}

#[cfg(feature = "leptos")]
#[component]
fn Foo() -> impl IntoView {
    Timeout::new(2000, api::command::emit).forget();

    let foo = TestState::use_field::<test_mod::Foo>("Test".into());

    view! { <h1>{foo}</h1> }
}
