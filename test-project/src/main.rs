#![allow(clippy::disallowed_names)]

use api::event::Listen;
use api::model::{test_mod, TestState};
use gloo_timers::callback::Timeout;
#[cfg(feature = "leptos")]
use leptos::{component, view, IntoView};

fn main() {
    console_log::init_with_level(log::Level::Trace).expect("no errors during logger init");
    console_error_panic_hook::set_once();

    api::cmd::empty_invoke();
    api::cmd::underscore_invoke(69);

    wasm_bindgen_futures::spawn_local(async {
        log::info!("{}", api::cmd::greet("frontend").await);

        api::cmd::await_heavy_computing().await;
        log::info!("heavy computing finished")
    });

    wasm_bindgen_futures::spawn_local(async move {
        let handle_bar = TestState::listen_to::<test_mod::FBar>(|echo| log::info!("bar: {echo}"))
            .await
            .unwrap();

        // with the move here, we hold "handle" in scope... if we wouldn't do that
        // handle would be dropped already and we get errors that the closure isn't anymore in scope
        //
        // it can be fixed with `handle.closure.take().unwrap().forget()`
        // see the `Closure::forget` docs, why this isn't the recommended way
        Timeout::new(2000, move || drop(handle_bar)).forget();
    });

    Timeout::new(1000, api::cmd::emit).forget();
    Timeout::new(3000, api::cmd::emit).forget();

    #[cfg(feature = "leptos")]
    Timeout::new(5000, || leptos::mount_to_body(|| view! { <App /> })).forget();
}

#[cfg(feature = "leptos")]
#[component]
fn App() -> impl IntoView {
    use leptos::SignalGet;

    let bar = TestState::use_field::<test_mod::FBar>(Some(true));

    let exit = move |_| api::model::other_cmd::stop_application();

    view! {
        <div>
            <button on:click=exit>Exit</button>
            <Foo/>
            {move || if bar.get() {
                Foo.into_view()
            } else {
                Foo.into_view()
            }}
        </div>
    }
}

#[cfg(feature = "leptos")]
#[component]
fn Foo() -> impl IntoView {
    log::info!("create foo");
    Timeout::new(3000, || {
        log::info!("emit foo");
        api::cmd::emit();
    }).forget();

    let foo = TestState::use_field::<test_mod::FFoo>(None);

    view! { <h1>{foo}</h1> }
}
