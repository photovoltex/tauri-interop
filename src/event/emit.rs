use tauri::{AppHandle, Error, Wry};

use super::Field;
#[cfg(doc)]
use super::Listen;

#[cfg(feature = "initial_value")]
#[doc(cfg(feature = "initial_value"))]
/// A mod containing functions to acquire a wrapped state manged by tauri
pub mod state_helper;

/// The trait which needs to be implemented for a [Field]
///
/// Conditionally changes between [Listen] and [Emit] or [ManagedEmit]
///
/// - When compiled to "target_family = wasm", the trait alias is set to [Listen]
/// - When feature "initial_value" is enabled, the trait alias is set to [ManagedEmit]
/// - Otherwise the trait alias is set to [Emit]
#[cfg(any(not(feature = "initial_value"), doc))]
pub trait Parent = Emit;

/// The trait which needs to be implemented for a [Field]
#[cfg(all(feature = "initial_value", not(doc)))]
pub trait Parent = ManagedEmit;

/// Extension of [Emit] to additionally require [Self] to be managed by tauri
#[cfg(feature = "initial_value")]
#[doc(cfg(feature = "initial_value"))]
pub trait ManagedEmit: Emit
    where
        Self: 'static + Send + Sync,
{
    /// Gets the value of a [Field] from [AppHandle]
    ///
    /// The default implementation acquires [Self] directly using [state_helper::directly].
    /// Override the provided method when [Self] is not directly managed. For example,
    /// this could be the case when the [interior mutability](https://doc.rust-lang.org/reference/interior-mutability.html)
    /// pattern is used to allow mutation of [Self] while being managed by tauri.
    ///
    /// Default state acquiring is provided via [state_helper].
    fn get_value<F: Field<Self>>(
        handle: &AppHandle,
        f: impl Fn(&Self) -> F::Type,
    ) -> Option<F::Type> {
        state_helper::directly::<Self, F>(handle, f)
    }
}

/// Trait that defines the available event emitting methods
pub trait Emit: Sized {
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
    /// #[cfg(feature = "initial_value")]
    /// impl tauri_interop::event::ManagedEmit for Test {}
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
    /// #[cfg(feature = "initial_value")]
    /// impl tauri_interop::event::ManagedEmit for Test {}
    ///
    /// #[tauri_interop::command]
    /// fn emit_bar(handle: TauriAppHandle) {
    ///     Test::default().emit::<test::FFoo>(&handle).expect("emitting failed");
    /// }
    ///
    /// fn main() {}
    /// ```
    fn emit<F: Field<Self>>(&self, handle: &AppHandle<Wry>) -> Result<(), Error>
        where
            Self: Parent;

    /// Update a single field and emit it afterward
    ///
    /// ### Example
    ///
    /// ```
    /// use tauri_interop::{command::TauriAppHandle, Event, event::Emit};
    ///
    ///
    /// #[derive(Default, Event)]
    /// pub struct Test {
    ///     foo: String,
    ///     pub bar: bool,
    /// }
    ///
    /// #[cfg(feature = "initial_value")]
    /// impl tauri_interop::event::ManagedEmit for Test {}
    ///
    /// #[tauri_interop::command]
    /// fn emit_bar(handle: TauriAppHandle) {
    ///     Test::default().update::<test::FBar>(&handle, true).expect("emitting failed");
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
            Self: Parent;
}
