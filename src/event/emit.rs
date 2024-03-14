use tauri::{AppHandle, Error, Wry};

use super::Field;
#[cfg(doc)]
use super::Listen;

/// The trait which needs to be implemented for a [Field]
///
/// Conditionally changes between [Listen] and [Emit]
///
/// - When compiled to "target_family = wasm", the trait alias is set to [Listen]
/// - Otherwise the trait alias is set to [Emit]
#[cfg(any(not(feature = "initial_value"), doc))]
pub trait Parent = Emit;

/// Trait that defines the available event emitting methods
pub trait Emit {
    /// Emit all field events
    ///
    /// ### Example
    ///
    /// ```
    /// use tauri_interop::{command::TauriAppHandle, event::Emit, Event};
    ///
    /// #[derive(Default, Event)]
    /// pub struct Test {
    ///     foo: String,
    ///     pub bar: bool,
    /// }
    ///
    /// #[tauri_interop::command]
    /// fn emit_bar(handle: TauriAppHandle) {
    ///     Test::default().emit_all(&handle).expect("emitting failed");
    /// }
    ///
    /// fn main() {}
    /// ```
    fn emit_all(&self, handle: &AppHandle<Wry>) -> Result<(), Error>;

    /// Emit a single field event
    ///
    /// ### Example
    ///
    /// ```
    /// use tauri_interop::{command::TauriAppHandle, event::Emit, Event};
    ///
    /// #[derive(Default, Event)]
    /// pub struct Test {
    ///     foo: String,
    ///     pub bar: bool,
    /// }
    ///
    /// #[tauri_interop::command]
    /// fn emit_bar(handle: TauriAppHandle) {
    ///     Test::default().emit::<test::Foo>(&handle).expect("emitting failed");
    /// }
    ///
    /// fn main() {}
    /// ```
    fn emit<F: Field<Self>>(&self, handle: &AppHandle<Wry>) -> Result<(), Error>
    where
        Self: Sized + Emit;

    /// Update a single field and emit it afterward
    ///
    /// ### Example
    ///
    /// ```
    /// use tauri_interop::{command::TauriAppHandle, event::Emit, Event};
    ///
    /// #[derive(Default, Event)]
    /// pub struct Test {
    ///     foo: String,
    ///     pub bar: bool,
    /// }
    ///
    /// #[tauri_interop::command]
    /// fn emit_bar(handle: TauriAppHandle) {
    ///     Test::default().update::<test::Bar>(&handle, true).expect("emitting failed");
    /// }
    ///
    /// fn main() {}
    /// ```
    fn update<F: Field<Self>>(
        &mut self,
        handle: &AppHandle<Wry>,
        field: F::Type,
    ) -> Result<(), Error>
    where
        Self: Sized + Emit;
}
