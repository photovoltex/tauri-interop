use serde::Serialize;
use tauri::{AppHandle, Error, Wry};

/// Trait defining a [EmitField] to a related struct implementing [Emit] with the related [EmitField::Type]
pub trait EmitField<Parent>
where
    Parent: Emit,
    <Self as EmitField<Parent>>::Type: Serialize + Clone,
{
    /// The type of the field
    type Type;

    /// Emits event of the related field with their value
    fn emit(parent: &Parent, handle: &AppHandle<Wry>) -> Result<(), Error>;

    /// Updates the related field and emits it
    fn update(parent: &mut Parent, handle: &AppHandle<Wry>, v: Self::Type) -> Result<(), Error>;
}

/// Trait that defines the available event emitting methods
pub trait Emit {
    /// Emit all field events
    fn emit_all(&self, handle: &AppHandle<Wry>) -> Result<(), Error>;

    /// Emit a single field event
    fn emit<F: EmitField<Self>>(&self, handle: &AppHandle<Wry>) -> Result<(), Error>
    where
        Self: Sized;

    /// Update a single field and emit it afterward
    fn update<F: EmitField<Self>>(
        &mut self,
        handle: &AppHandle<Wry>,
        field: F::Type,
    ) -> Result<(), Error>
    where
        Self: Sized;
}
