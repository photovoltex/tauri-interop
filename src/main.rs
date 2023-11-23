use gloo_timers::callback::Timeout;

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("no errors during logger init");
    console_error_panic_hook::set_once();

    api::cmd::empty_invoke();

    log::info!("logger is initialized");

    wasm_bindgen_futures::spawn_local(async {
        log::info!("invoke_with_return: {}", api::cmd::invoke_with_return().await)
    });

    wasm_bindgen_futures::spawn_local(async move {
        let handle = api::model::listen_to_echo(|echo|
            log::info!("echo: {echo}")
        ).await.unwrap();

        Timeout::new(1000, api::cmd::echo).forget();
        // with the move here, we hold "handle" in scope... if we wouldn't do that
        // handle would be dropped already and we get errors in the ui
        //
        // it can be fixed with `handle.closure.take().unwrap().forget()`
        // see the `Closure::forget` docs, why this isn't the recommended way
        Timeout::new(2000, move || handle.detach_listen()).forget();
        Timeout::new(3000, api::cmd::echo).forget();

    });
}
