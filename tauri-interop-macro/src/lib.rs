use std::{collections::BTreeSet, sync::Mutex};

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, FnArg, Ident, ItemFn, ItemUse, Pat,
    PathSegment, ReturnType, Signature, Type,
};

#[cfg(feature = "listen")]
use syn::ItemStruct;

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

#[cfg(feature = "listen")]
#[proc_macro_attribute]
pub fn emit(_: TokenStream, stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);

    if stream_struct.fields.is_empty() {
        panic!("No fields provided")
    }

    if stream_struct.fields.iter().any(|field| field.ident.is_none()) {
        panic!("Tuple Structs aren't supported")
    }

    let name = format_ident!("{}Emit", stream_struct.ident);
    let variants = stream_struct
        .fields
        .iter()
        .map(|field| {
            let field_ident = field.ident.as_ref().expect("handled before");
            let variation = field_ident.to_string().to_case(Case::Pascal);

            (format_ident!("{field_ident}"), format_ident!("{variation}"))
        })
        .collect::<Vec<_>>();

    let struct_ident = &stream_struct.ident;
    let mapped_variants = variants
        .iter()
        .map(|(field_ident, variant_ident)| {
            // todo: had an thought, where we could replace this duplicate code with an enum... look later into plz :3
            quote! {
                #name::#variant_ident => {
                    let (struct_ident, field_ident) = (stringify!(#struct_ident), stringify!(#field_ident));
                    log::trace!("{struct_ident} emitted [{field_ident}] via provided handle");
                    handle.emit_all(field_ident, self.#field_ident.clone())
                }
            }
        })
        .collect::<Vec<_>>();

    let variants = variants
        .into_iter()
        .map(|(_, variation)| variation)
        .collect::<Vec<_>>();

    let stream = quote! {
        #[derive(Debug, Clone)]
        pub enum #name {
            #( #variants ),*
        }
        
        #stream_struct

        impl #struct_ident {
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

#[cfg(feature = "listen")]
#[proc_macro_attribute]
pub fn listen_to(_: TokenStream, stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);

    if stream_struct.fields.is_empty() {
        panic!("No fields provided")
    }

    if stream_struct.fields.iter().any(|field| field.ident.is_none()) {
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
            quote! {
                #[must_use = "If the returned handle is dropped, the contained closure goes out of scope and can't be called"]
                pub async fn #fn_name(callback: impl Fn(#ty) + 'static) -> ::tauri_interop::listen::ListenResult {
                    tauri_interop::listen::register_listener(stringify!(#field_ident), callback).await
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

const TAURI_TYPES: [&str; 3] = ["State", "AppHandle", "Window"];

/// really cheap filter for TAURI_TYPES
/// didn't figure out a way to only include tauri:: Structs/Enums and
/// for now all ident name like the above TAURI_TYPES are filtered
fn is_tauri_type(segment: &PathSegment) -> bool {
    TAURI_TYPES.contains(&segment.ident.to_string().as_str())
}

fn is_result(segment: &PathSegment) -> bool {
    segment.ident.to_string().as_str() == "Result"
}

/// wasm command
#[proc_macro_attribute]
pub fn command(_: TokenStream, stream: TokenStream) -> TokenStream {
    let ItemFn { attrs, sig, .. } = parse_macro_input!(stream as ItemFn);

    let Signature {
        ident,
        generics,
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
                    // fixme: if it's an single ident, catch isn't needed
                    // this could probably be a problem later
                    Type::Path(path) => path.path.segments.iter().any(is_result),
                    others => panic!("no support for '{}'", others.to_token_stream()),
                },
            )
        }
    };

    let mut args_inputs: Punctuated<Ident, Comma> = Punctuated::new();
    let wasm_inputs = inputs
        .into_iter()
        .filter_map(|fn_inputs| {
            if let FnArg::Typed(typed) = &fn_inputs {
                if let Type::Path(path) = typed.ty.as_ref() {
                    if path.path.segments.iter().any(is_tauri_type) {
                        return None;
                    }
                }

                if let Pat::Ident(ident) = typed.pat.as_ref() {
                    args_inputs.push(ident.ident.clone());
                    return Some(fn_inputs);
                }
            }
            None
        })
        .collect::<Punctuated<FnArg, Comma>>();

    let args_ident = format_ident!("{}Args", ident.to_string().to_case(Case::Pascal));

    let invoke = if need_catch {
        quote! {
            ::tauri_interop::bindings::invoke_catch(stringify!(#ident), args).await
                .map(|value| serde_wasm_bindgen::from_value(value).expect("ok: conversion error"))
                .map_err(|value| serde_wasm_bindgen::from_value(value).expect("err: conversion error"))
        }
    } else if async_ident.is_some() {
        quote! {
            let value = ::tauri_interop::bindings::async_invoke(stringify!(#ident), args).await;
            serde_wasm_bindgen::from_value(value).expect("conversion error")
        }
    } else {
        quote! {
            ::tauri_interop::bindings::invoke(stringify!(#ident), args);
        }
    };

    let stream = quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        struct #generics #args_ident {
            #wasm_inputs
        }

        #( #attrs )*
        pub #async_ident fn #ident #generics (#wasm_inputs) #variadic #output
        {
            let args = #args_ident { #args_inputs };
            let args = serde_wasm_bindgen::to_value(&args)
                .expect("serialized arguments");

            #invoke
        }
    };

    TokenStream::from(stream.to_token_stream())
}

/// command which is expected to be used instead of tauri::command
#[proc_macro_attribute]
pub fn conditional_command(_: TokenStream, stream: TokenStream) -> TokenStream {
    let fn_item = syn::parse::<ItemFn>(stream).unwrap();

    HANDLER_LIST
        .lock()
        .unwrap()
        .insert(fn_item.sig.ident.to_string());

    let command_macro = quote! {
        #[cfg_attr(target_family = "wasm", tauri_interop::command)]
        #[cfg_attr(not(target_family = "wasm"), tauri::command)]
        #fn_item
    };

    TokenStream::from(command_macro.to_token_stream())
}

/// collects all commands annotated with the `conditional_command`s macro
#[proc_macro]
pub fn collect_handlers(_: TokenStream) -> TokenStream {
    let handler = HANDLER_LIST.lock().unwrap();
    let handler = handler
        .iter()
        .map(|s| format_ident!("{s}"))
        .collect::<Punctuated<Ident, Comma>>();

    let stream = quote! {
        #[cfg(not(target_family = "wasm"))]
        /// the all mighty handler collector
        pub fn get_handlers() -> impl Fn(tauri::Invoke) {
            tauri::generate_handler![ #handler ]
        }
    };

    TokenStream::from(stream.to_token_stream())
}

#[proc_macro_attribute]
pub fn conditional_use(_: TokenStream, stream: TokenStream) -> TokenStream {
    let item_use = parse_macro_input!(stream as ItemUse);

    let command_macro = quote! {
        #[cfg(not(target_family = "wasm"))]
        #item_use
    };

    TokenStream::from(command_macro.to_token_stream())
}
