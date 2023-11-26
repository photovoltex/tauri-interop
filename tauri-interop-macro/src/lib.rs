#![warn(missing_docs)]
//! The macros use by `tauri_interop` to generate dynamic code depending on the target

#[cfg(feature = "listen")]
use std::fmt::Display;
use std::{collections::BTreeSet, sync::Mutex};

use convert_case::{Case, Casing};
use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, FnArg, Ident, ItemFn, ItemUse,
    Lifetime, LifetimeParam, Pat, PathSegment, ReturnType, Signature, Type,
};

#[cfg(feature = "listen")]
use syn::ItemStruct;

/// Conditionally adds [macro@listen_to] or [macro@emit] to a struct
#[cfg(feature = "listen")]
#[proc_macro_attribute]
pub fn emit_or_listen(_: TokenStream, stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);
    let stream = quote! {
        #[cfg_attr(target_family = "wasm", tauri_interop::listen_to)]
        #[cfg_attr(not(target_family = "wasm"), tauri_interop::emit)]
        #stream_struct
    };

    TokenStream::from(stream.to_token_stream())
}

/// function to build the same unique event name for wasm and host triplet
#[cfg(feature = "listen")]
fn get_event_name<S, F>(struct_name: &S, field_name: &F) -> String
where
    S: Display,
    F: Display,
{
    format!("{struct_name}::{field_name}")
}

/// Generates an `emit` function for the given struct with a
/// correlation enum for emitting a single field of the struct.
///
/// Used for host code generation.
#[cfg(feature = "listen")]
#[proc_macro_attribute]
pub fn emit(_: TokenStream, stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);

    if stream_struct.fields.is_empty() {
        panic!("No fields provided")
    }

    if stream_struct
        .fields
        .iter()
        .any(|field| field.ident.is_none())
    {
        panic!("Tuple Structs aren't supported")
    }

    let name = format_ident!("{}Emit", stream_struct.ident);
    let variants = stream_struct
        .fields
        .iter()
        .map(|field| {
            let field_ident = field.ident.as_ref().expect("handled before");
            let variation = field_ident.to_string().to_case(Case::Pascal);

            (field_ident, format_ident!("{variation}"), &field.ty)
        })
        .collect::<Vec<_>>();

    let struct_ident = &stream_struct.ident;
    let mut updaters = Vec::new();
    let mapped_variants = variants
        .iter()
        .map(|(field_ident, variant_ident, ty)| {
            let update = format_ident!("update_{}", field_ident);
            updaters.push(quote!{
                pub fn #update(&mut self, handle: &tauri::AppHandle, #field_ident: #ty) -> Result<(), tauri::Error> {
                    self.#field_ident = #field_ident;
                    self.emit(handle, #name::#variant_ident)
                }
            });

            let event_name = get_event_name(struct_ident, field_ident);

            quote! {
                #name::#variant_ident => {
                    log::trace!(
                        "{} emitted [{}] via provided handle",
                        stringify!(#struct_ident),
                        stringify!(#field_ident),
                    );
                    handle.emit_all(#event_name, self.#field_ident.clone())
                }
            }
        })
        .collect::<Vec<_>>();

    let variants = variants
        .into_iter()
        .map(|(_, variation, _)| variation)
        .collect::<Vec<_>>();

    let stream = quote! {
        #[derive(Debug, Clone)]
        pub enum #name {
            #( #variants ),*
        }

        #stream_struct

        impl #struct_ident {
            #( #updaters )*

            #[must_use]
            pub fn emit(&self, handle: &::tauri::AppHandle, field: #name) -> Result<(), tauri::Error> {
                use tauri::Manager;

                match field {
                    #( #mapped_variants ),*
                }
            }
        }
    };

    TokenStream::from(stream.to_token_stream())
}

/// Generates `listen_to_<field>` functions for the given
/// struct for the correlating host code.
///
/// Used for wasm code generation
#[cfg(feature = "listen")]
#[proc_macro_attribute]
pub fn listen_to(_: TokenStream, stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);

    if stream_struct.fields.is_empty() {
        panic!("No fields provided")
    }

    if stream_struct
        .fields
        .iter()
        .any(|field| field.ident.is_none())
    {
        panic!("Tuple Structs aren't supported")
    }

    let struct_ident = &stream_struct.ident;

    let mapped_variants = stream_struct
        .fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            let field_ident = field
                .ident
                .as_ref()
                .expect("handled before")
                .clone();
            let fn_ident = field_ident.to_string().to_case(Case::Snake).to_lowercase();
            let fn_name = format_ident!("listen_to_{fn_ident}");

            let event_name = get_event_name(struct_ident, &field_ident);

            quote! {
                #[must_use = "If the returned handle is dropped, the contained closure goes out of scope and can't be called"]
                pub async fn #fn_name<'s>(callback: impl Fn(#ty) + 'static) -> ::tauri_interop::listen::ListenResult<'s> {
                    ::tauri_interop::listen::ListenHandle::register(#event_name, callback).await
                }
            }
        }).collect::<Vec<_>>();

    let stream = quote! {
        #stream_struct

        impl #struct_ident {
            #( #mapped_variants )*
        }
    };

    TokenStream::from(stream.to_token_stream())
}

lazy_static::lazy_static! {
    static ref HANDLER_LIST: Mutex<BTreeSet<String>> = Mutex::new(Default::default());
}

const ARGUMENT_LIFETIME: &str = "'arg_lifetime";
const TAURI_TYPES: [&str; 3] = ["State", "AppHandle", "Window"];

/// really cheap filter for TAURI_TYPES
///
/// didn't figure out a way to only include tauri:: Structs/Enums and
/// for now all ident name like the above TAURI_TYPES are filtered
fn is_tauri_type(segment: &PathSegment) -> bool {
    TAURI_TYPES.contains(&segment.ident.to_string().as_str())
}

/// simple filter for determining if the given path is a [Result]
fn is_result(segment: &PathSegment) -> bool {
    segment.ident.to_string().as_str() == "Result"
}

/// Generates the wasm counterpart to a defined `tauri::command`
#[proc_macro_attribute]
pub fn binding(_: TokenStream, stream: TokenStream) -> TokenStream {
    let ItemFn { attrs, sig, .. } = parse_macro_input!(stream as ItemFn);

    let Signature {
        ident,
        mut generics,
        inputs,
        variadic,
        output,
        ..
    } = sig;

    let (async_ident, need_catch) = match &output {
        ReturnType::Default => (None, false),
        ReturnType::Type(_, ty) => {
            (
                Some(format_ident!("async")),
                match ty.as_ref() {
                    // fixme: if it's an single ident, catch isn't needed this could probably be a problem later
                    Type::Path(path) => path.path.segments.iter().any(is_result),
                    others => panic!("no support for '{}'", others.to_token_stream()),
                },
            )
        }
    };

    let mut requires_lifetime_constrain = false;
    let mut args_inputs: Punctuated<Ident, Comma> = Punctuated::new();
    let wasm_inputs = inputs
        .into_iter()
        .filter_map(|mut fn_inputs| {
            if let FnArg::Typed(ref mut typed) = fn_inputs {
                match typed.ty.as_mut() {
                    Type::Path(path) if path.path.segments.iter().any(is_tauri_type) => {
                        return None
                    }
                    Type::Reference(reference) => {
                        reference.lifetime =
                            Some(Lifetime::new(ARGUMENT_LIFETIME, Span::call_site().into()));
                        requires_lifetime_constrain = true;
                    }
                    _ => {}
                }

                if let Pat::Ident(ident) = typed.pat.as_ref() {
                    args_inputs.push(ident.ident.clone());
                    return Some(fn_inputs);
                }
            }
            None
        })
        .collect::<Punctuated<FnArg, Comma>>();

    if requires_lifetime_constrain {
        let lt = Lifetime::new(ARGUMENT_LIFETIME, Span::call_site().into());
        generics
            .params
            .push(syn::GenericParam::Lifetime(LifetimeParam::new(lt)))
    }

    let invoke = if need_catch {
        quote!(::tauri_interop::command::invoke_catch(stringify!(#ident), args).await)
    } else if async_ident.is_some() {
        quote!(::tauri_interop::command::async_invoke(stringify!(#ident), args).await)
    } else {
        quote!(::tauri_interop::bindings::invoke(stringify!(#ident), args);)
    };

    let args_ident = format_ident!("{}Args", ident.to_string().to_case(Case::Pascal));
    let stream = quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        struct #args_ident #generics {
            #wasm_inputs
        }

        #( #attrs )*
        pub #async_ident fn #ident #generics (#wasm_inputs) #variadic #output
        {
            let args = #args_ident { #args_inputs };
            let args = ::serde_wasm_bindgen::to_value(&args)
                .expect("serialized arguments");

            #invoke
        }
    };

    TokenStream::from(stream.to_token_stream())
}

/// Conditionally adds [macro@binding] or `tauri::command` to a struct
#[proc_macro_attribute]
pub fn command(_: TokenStream, stream: TokenStream) -> TokenStream {
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

/// Collects all commands annotated with [macro@command] and
/// provides these with a `get_handlers()` in the current namespace
#[proc_macro]
pub fn collect_commands(_: TokenStream) -> TokenStream {
    let handler = HANDLER_LIST.lock().unwrap();
    let handler = handler
        .iter()
        .map(|s| format_ident!("{s}"))
        .collect::<Punctuated<Ident, Comma>>();

    let stream = quote! {
        #[cfg(not(target_family = "wasm"))]
        /// the all mighty handler collector
        pub fn get_handlers() -> impl Fn(tauri::Invoke) {
            ::tauri::generate_handler![ #handler ]
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
