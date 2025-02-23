use std::sync::RwLock;

use tauri::{AppHandle, Manager};
use tauri_interop::event::{Field, ManagedEmit};

impl ManagedEmit for super::TestState {
    fn get_value<F: Field<Self>>(
        handle: &AppHandle,
        get_field_value: impl Fn(&Self) -> F::Type,
    ) -> Option<F::Type> {
        let state = handle.try_state::<RwLock<Self>>()?;
        let state = state.read().ok()?;
        Some(get_field_value(&state))
    }
}

impl ManagedEmit for super::NamingTestEnum {}
impl ManagedEmit for super::NamingTestDefault {}
