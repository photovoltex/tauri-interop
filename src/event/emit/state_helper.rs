use std::sync::{Mutex, RwLock};
use super::*;

/// Acquires the state directly
///
/// Default usage when [ManagedEmit::get_value] isn't overridden.
pub fn directly<P: ManagedEmit, F: Field<P>>(
    handle: &AppHandle,
    f: impl Fn(&P) -> F::Type,
) -> Option<F::Type> {
    use tauri::Manager;

    let state = handle.try_state::<P>()?;
    Some(f(&state))
}

/// Acquires the state wrapped in an [Option]
pub fn option<P: ManagedEmit, F: Field<P>>(
    handle: &AppHandle,
    f: impl Fn(&P) -> F::Type,
) -> Option<F::Type> {
    use tauri::Manager;

    let state = handle.try_state::<Option<P>>()?;
    Some(f(state.as_ref()?))
}

/// Acquires the state wrapped in an [RwLock]
pub fn rwlock<P: ManagedEmit, F: Field<P>>(
    handle: &AppHandle,
    f: impl Fn(&P) -> F::Type,
) -> Option<F::Type> {
    use tauri::Manager;

    let state = handle.try_state::<RwLock<P>>()?;
    let state = state.read().ok()?;
    Some(f(&state))
}

/// Acquires the state wrapped in a [Mutex]
pub fn mutex<P: ManagedEmit, F: Field<P>>(
    handle: &AppHandle,
    f: impl Fn(&P) -> F::Type,
) -> Option<F::Type> {
    use tauri::Manager;

    let state = handle.try_state::<Mutex<P>>()?;
    let state = state.lock().ok()?;
    Some(f(&state))
}
