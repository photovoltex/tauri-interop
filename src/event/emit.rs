use tauri::{AppHandle, Error, Wry};

use super::Field;

/// Trait that defines the available event emitting methods
pub trait Emit {
    /// Emit all field events
    fn emit_all(&self, handle: &AppHandle<Wry>) -> Result<(), Error>;

    /// Emit a single field event
    fn emit<F: Field<Self>>(&self, handle: &AppHandle<Wry>) -> Result<(), Error>
    where
        Self: Sized + Emit;

    /// Update a single field and emit it afterward
    fn update<F: Field<Self>>(
        &mut self,
        handle: &AppHandle<Wry>,
        field: F::Type,
    ) -> Result<(), Error>
    where
        Self: Sized + Emit;
}
