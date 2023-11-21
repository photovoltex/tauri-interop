use api::listen::{ListenHandle, ListenError};
use gloo_timers::callback::Timeout;
use js_sys::Function;
use serde::{Serialize, Deserialize};
use wasm_bindgen::{closure::Closure, JsCast};

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("no errors during logger init");
    console_error_panic_hook::set_once();

    api::cmd::empty_invoke();

    wasm_bindgen_futures::spawn_local(async {
        log::info!("test");
        log::info!("{}", api::cmd::invoke_with_return().await)
    });

    wasm_bindgen_futures::spawn_local(async move {
        let handle = listen_to_echo(|echo| log::info!("{echo}")).await.unwrap();

        Timeout::new(1000, api::cmd::echo).forget();
        Timeout::new(2000, move || handle.detach_listen()).forget();
        Timeout::new(3000, api::cmd::echo).forget();

    });
}

#[derive(Debug, Serialize, Deserialize)]
struct PayloadEcho {
    payload: String,
    event: String
}

async fn listen_to_echo(callback: impl Fn(String) + 'static) -> Result<ListenHandle, ListenError> {
    let closure = Closure::new(move |value| {
        let payload = serde_wasm_bindgen::from_value::<PayloadEcho>(value).expect("serializable");
        callback(payload.payload)
    });

    let ignore = api::bindings::listen("echo", &closure);
    let detach_fn = wasm_bindgen_futures::JsFuture::from(ignore)
        .await
        .map_err(|_| ListenError::PromiseFailed)?
        .dyn_into::<Function>()
        .map_err(|_| ListenError::NotAFunction)?;

    Ok(ListenHandle::new(closure, detach_fn))
}
