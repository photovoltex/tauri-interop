use std::fmt::Debug;

use serde::Serialize;
use tauri::{AppHandle, Error, Wry};

/// Trait intended to mark an enum as usable [Emit::Fields]
pub trait EmitFields: Debug {}

/// Trait defining a [EmitField] to a related struct implementing [Emit] with the related [EmitField::Type]
pub trait EmitField<S>
where
    S: Emit,
    <S as Emit>::Fields: EmitFields,
    <Self as EmitField<S>>::Type: Serialize + Clone,
{
    /// The type of the field
    type Type;

    /// Updates the related field and emit it's event
    fn update(s: &mut S, handle: &AppHandle<Wry>, v: Self::Type) -> Result<(), Error>;
}

/// Trait that defines the available event emitting methods
pub trait Emit
where
    <Self as Emit>::Fields: EmitFields,
{
    /// Fields that are used to emit an event
    type Fields;

    /// Emit a single field event
    fn emit(&self, handle: &AppHandle<Wry>, field: Self::Fields) -> Result<(), Error>;
    /// Emit all field events
    fn emit_all(&self, handle: &AppHandle<Wry>) -> Result<(), Error>;
    /// Emit and update a single field
    fn update<F: EmitField<Self>>(
        &mut self,
        handle: &AppHandle<Wry>,
        field: F::Type,
    ) -> Result<(), Error>
    where
        Self: Sized;
}
