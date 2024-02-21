#![warn(missing_docs)]
//! The macros use by `tauri_interop` to generate dynamic code depending on the target

mod command;
mod event;

use std::{collections::BTreeSet, sync::Mutex};

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Ident, ItemFn, ItemUse};

/// Conditionally adds [Listen] or [Emit] to a struct
#[cfg(feature = "event")]
#[proc_macro_derive(Event)]
pub fn derive_event(stream: TokenStream) -> TokenStream {
    if cfg!(feature = "wasm") {
        event::listen::derive(stream)
    } else {
        event::emit::derive(stream)
    }
}

/// Generates a default `Emit` implementation for the given struct with a
/// correlation enum, mod and field-structs for emitting a single field of
/// the struct.
///
/// Used for host code generation.
#[cfg(feature = "event")]
#[proc_macro_derive(Emit)]
pub fn derive_emit(stream: TokenStream) -> TokenStream {
    event::emit::derive(stream)
}

/// Generates a default `EmitField` implementation for the given struct.
///
/// Used for host code generation.
#[cfg(feature = "event")]
#[proc_macro_derive(EmitField, attributes(parent, parent_field_name, parent_field_ty))]
pub fn derive_emit_field(stream: TokenStream) -> TokenStream {
    event::emit::derive_field(stream)
}

/// Generates `listen_to_<field>` functions for the given
/// struct for the correlating host code.
///
/// Used for wasm code generation
#[cfg(feature = "event")]
#[proc_macro_derive(Listen)]
pub fn derive_listen(stream: TokenStream) -> TokenStream {
    event::listen::derive(stream)
}

/// Generates a default `ListenField` implementation for the given struct.
///
/// Used for wasm code generation.
#[cfg(feature = "event")]
#[proc_macro_derive(ListenField, attributes(parent, parent_field_ty))]
pub fn derive_listen_field(stream: TokenStream) -> TokenStream {
    event::listen::derive_field(stream)
}

/// Generates the wasm counterpart to a defined `tauri::command`
#[proc_macro_attribute]
pub fn binding(_attributes: TokenStream, stream: TokenStream) -> TokenStream {
    command::convert_to_binding(stream)
}

lazy_static::lazy_static! {
    static ref HANDLER_LIST: Mutex<BTreeSet<String>> = Mutex::new(Default::default());
}

/// Conditionally adds `tauri_interop::binding` or `tauri::command` to a struct
#[proc_macro_attribute]
pub fn command(_attributes: TokenStream, stream: TokenStream) -> TokenStream {
    let fn_item = syn::parse::<ItemFn>(stream).unwrap();

    HANDLER_LIST
        .lock()
        .unwrap()
        .insert(fn_item.sig.ident.to_string());

    let command_macro = quote! {
        #[cfg_attr(target_family = "wasm", tauri_interop::binding)]
        #[cfg_attr(not(target_family = "wasm"), tauri::command(rename_all = "snake_case"))]
        #fn_item
    };

    TokenStream::from(command_macro.to_token_stream())
}

/// Collects all commands annotated with `tauri_interop::command` and
/// provides these with a `get_handlers()` in the current namespace
///
/// The provided function isn't available for wasm
#[proc_macro]
pub fn collect_commands(_: TokenStream) -> TokenStream {
    let handler = HANDLER_LIST.lock().unwrap();
    let to_generated_handler = handler
        .iter()
        .map(|s| format_ident!("{s}"))
        .collect::<Punctuated<Ident, Comma>>();

    let stream = quote! {
        #[cfg(not(target_family = "wasm"))]
        /// the all mighty handler collector
        pub fn get_handlers() -> impl Fn(tauri::Invoke) {
            let handlers = vec! [ #( #handler ),* ];
            log::debug!("Registering following commands to tauri: {handlers:#?}");

            ::tauri::generate_handler![ #to_generated_handler ]
        }
    };

    TokenStream::from(stream.to_token_stream())
}

/// Simple macro to include given `use` only in host
#[proc_macro_attribute]
pub fn host_usage(_: TokenStream, stream: TokenStream) -> TokenStream {
    let item_use = parse_macro_input!(stream as ItemUse);

    let command_macro = quote! {
        #[cfg(not(target_family = "wasm"))]
        #item_use
    };

    TokenStream::from(command_macro.to_token_stream())
}

/// Simple macro to include given `use` only in wasm
#[proc_macro_attribute]
pub fn wasm_usage(_: TokenStream, stream: TokenStream) -> TokenStream {
    let item_use = parse_macro_input!(stream as ItemUse);

    let command_macro = quote! {
        #[cfg(target_family = "wasm")]
        #item_use
    };

    TokenStream::from(command_macro.to_token_stream())
}
