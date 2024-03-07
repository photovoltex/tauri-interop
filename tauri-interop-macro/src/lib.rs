#![feature(iter_intersperse)]
#![warn(missing_docs)]
//! The macros use by `tauri_interop` to generate dynamic code depending on the target

use proc_macro::TokenStream;
use std::collections::HashSet;
use std::sync::Mutex;

use proc_macro_error::{emit_call_site_error, emit_call_site_warning, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, ExprPath, ItemFn, ItemMod, Token,
};

use crate::command::collect::commands_to_punctuated;

mod command;
mod event;

/// Conditionally adds [Listen] or [Emit] to a struct.
///
/// The field values inside the struct require to be self owned.
/// That means references aren't allowed inside the event struct.
///
/// Depending on the targeted architecture the macro generates
/// different results. When compiling to `wasm` the [Listen] trait is
/// derived. Otherwise, [Emit] is derived.
///
/// Both traits generate a new mod in which the related field-structs
/// are generated in. The field-struct represent a field of the struct
/// and are used for the derived trait functions.
///
/// ### Example
///
/// ```
/// use tauri_interop_macro::Event;
///
/// #[derive(Event)]
/// struct EventModel {
///     foo: String,
///     pub bar: bool
/// }
///
/// // has to be defined in this example, otherwise the
/// // macro expansion panics because of missing super
/// fn main() {}
/// ```
#[cfg(feature = "event")]
#[proc_macro_derive(Event, attributes(auto_naming, mod_name))]
pub fn derive_event(stream: TokenStream) -> TokenStream {
    if cfg!(feature = "_wasm") {
        event::listen::derive(stream)
    } else {
        event::emit::derive(stream)
    }
}

/// Generates a default `Emit` implementation for the given struct.
///
/// Used for host code generation. It is not intended to be used directly.
/// See [Event]
#[cfg(feature = "event")]
#[proc_macro_derive(Emit, attributes(auto_naming, mod_name))]
pub fn derive_emit(stream: TokenStream) -> TokenStream {
    event::emit::derive(stream)
}

/// Generates a default `EmitField` implementation for the given struct.
///
/// Used for host code generation. It is not intended to be used directly.
#[cfg(feature = "event")]
#[proc_macro_derive(EmitField, attributes(parent, parent_field_name, parent_field_ty))]
pub fn derive_emit_field(stream: TokenStream) -> TokenStream {
    event::emit::derive_field(stream)
}

/// Generates `listen_to_<field>` functions for the given struct.
///
/// Used for wasm code generation. It is not intended to be used directly.
#[cfg(feature = "event")]
#[proc_macro_derive(Listen, attributes(auto_naming, mod_name))]
pub fn derive_listen(stream: TokenStream) -> TokenStream {
    event::listen::derive(stream)
}

/// Generates a default `ListenField` implementation for the given struct.
///
/// Used for wasm code generation. It is not intended to be used directly.
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
    static ref COMMAND_LIST_ALL: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

lazy_static::lazy_static! {
    static ref COMMAND_LIST: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

static COMMAND_MOD_NAME: Mutex<Option<String>> = Mutex::new(None);

/// Conditionally adds the [binding] or `tauri::command` macro to a struct
#[proc_macro_attribute]
pub fn command(_attributes: TokenStream, stream: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(stream as ItemFn);

    COMMAND_LIST
        .lock()
        .unwrap()
        .insert(fn_item.sig.ident.to_string());

    let command_macro = quote! {
        #[cfg_attr(target_family = "wasm", ::tauri_interop::binding)]
        #[cfg_attr(not(target_family = "wasm"), tauri::command(rename_all = "snake_case"))]
        #fn_item
    };

    TokenStream::from(command_macro.to_token_stream())
}

/// Marks a mod that contains commands
///
/// A mod needs to be marked when multiple command mods should be combined.
/// See [combine_handlers!] for a detailed explanation/example.
///
/// Requires usage of unstable feature: `#![feature(proc_macro_hygiene)]`
#[proc_macro_attribute]
pub fn commands(_attributes: TokenStream, stream: TokenStream) -> TokenStream {
    let item_mod = parse_macro_input!(stream as ItemMod);
    let _ = COMMAND_MOD_NAME
        .lock()
        .unwrap()
        .insert(item_mod.ident.to_string());

    TokenStream::from(item_mod.to_token_stream())
}

/// Collects all commands annotated with `tauri_interop::command` and
/// provides these with a `get_handlers()` in the current mod
///
/// The provided function isn't available in wasm
///
/// ### Example
///
/// ```
/// #[tauri_interop_macro::command]
/// fn greet(name: &str) -> String {
///     format!("Hello, {}! You've been greeted from Rust!", name)
/// }
///
/// tauri_interop_macro::collect_commands!();
///
/// fn main() {
///     let _ = tauri::Builder::default()
///     // This is where you pass in the generated handler collector
///     // in this example this would only register cmd1
///         .invoke_handler(get_handlers());
/// }
/// ```
#[proc_macro]
pub fn collect_commands(_: TokenStream) -> TokenStream {
    let mut commands = COMMAND_LIST.lock().unwrap();
    let stream = command::collect::get_handler_function(
        format_ident!("get_handlers"),
        &commands,
        commands_to_punctuated(&commands),
        Vec::new(),
    );

    // logic for renaming the commands, so that combine methode can just use the provided commands
    if let Some(mod_name) = COMMAND_MOD_NAME.lock().unwrap().as_ref() {
        COMMAND_LIST_ALL
            .lock()
            .unwrap()
            .extend(command::collect::commands_with_mod_name(
                mod_name, &commands,
            ));
    } else {
        // if there is no mod provided we can just move/clear the commands
        COMMAND_LIST_ALL
            .lock()
            .unwrap()
            .extend(commands.iter().cloned());
    }

    // clearing the already used handlers
    commands.clear();
    // set mod name to none
    let _ = COMMAND_MOD_NAME.lock().unwrap().take();

    TokenStream::from(stream.to_token_stream())
}

/// Combines multiple modules containing commands
///
/// Takes multiple module paths as input and provides a `get_all_handlers()` function in
/// the current mod that registers all commands from the provided mods. This macro does
/// still require the invocation of [collect_commands!] at the end of a command mod. In
/// addition, a mod has to be marked with [macro@commands].
///
/// ### Example
///
/// ```
/// #[tauri_interop_macro::commands]
/// mod cmd1 {
///     #[tauri_interop_macro::command]
///     pub fn cmd1() {}
///
///     tauri_interop_macro::collect_commands!();
/// }
///
/// mod whatever {
///     #[tauri_interop_macro::commands]
///     pub mod cmd2 {
///         #[tauri_interop_macro::command]
///         pub fn cmd2() {}
///
///         tauri_interop_macro::collect_commands!();
///     }
/// }
///
/// tauri_interop_macro::combine_handlers!( cmd1, whatever::cmd2 );
///
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn combine_handlers(stream: TokenStream) -> TokenStream {
    let command_mods = Punctuated::<ExprPath, Token![,]>::parse_terminated
        .parse2(stream.into())
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();

    let org_commands = COMMAND_LIST_ALL.lock().unwrap();
    let commands = command::collect::get_filtered_commands(&org_commands, &command_mods);

    if commands.is_empty() {
        emit_call_site_error!("No commands will be registered")
    }

    let remaining_commands = COMMAND_LIST.lock().unwrap();
    if !remaining_commands.is_empty() {
        emit_call_site_error!(
            "Their are dangling commands that won't be registered. See {:?}",
            remaining_commands
        )
    }

    if org_commands.len() > commands.len() {
        let diff = org_commands
            .difference(&commands)
            .cloned()
            .intersperse(String::from(","))
            .collect::<String>();
        emit_call_site_warning!(
            "Not all commands will be registered. Missing commands: {:?}",
            diff
        );
    }

    TokenStream::from(command::collect::get_handler_function(
        format_ident!("get_all_handlers"),
        &commands,
        commands_to_punctuated(&commands),
        command_mods,
    ))
}

/// Simple macro to include multiple imports (seperated by `|`) not in wasm
///
/// ### Example
///
/// ```rust
/// tauri_interop_macro::host_usage! {
///     use tauri::State;
///     | use std::sync::RwLock;
/// }
///
/// #[tauri_interop_macro::command]
/// pub fn empty_invoke(_state: State<RwLock<String>>) {}
/// ```
#[proc_macro]
pub fn host_usage(stream: TokenStream) -> TokenStream {
    let uses = command::collect::uses(stream);
    TokenStream::from(quote! {
        #(
            #[cfg(not(target_family = "wasm"))]
            #uses
        )*
    })
}

/// Simple macro to include multiple imports (seperated by `|`) only in wasm
///
/// Equivalent to [host_usage!] for wasm imports only required in wasm.
/// For an example see [host_usage!].
#[proc_macro]
pub fn wasm_usage(stream: TokenStream) -> TokenStream {
    let uses = command::collect::uses(stream);
    TokenStream::from(quote! {
        #(
            #[cfg(target_family = "wasm")]
            #uses
        )*
    })
}
