use std::{collections::BTreeSet, sync::Mutex};

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    punctuated::Punctuated, token::Comma, FnArg,
    Ident, ItemFn, Pat, ReturnType, Signature, Type, ItemUse, PathSegment
};

#[cfg(feature = "listen")]
#[proc_macro_attribute]
pub fn handle_emit_all(_: TokenStream, mut stream: TokenStream) -> TokenStream {
    let cp_stream = stream.clone();
    let input_enum = parse_macro_input!(cp_stream as ItemEnum);

    let ident = input_enum.ident;
    let mapped_variants = input_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = variant.ident.clone();

            if variant.fields.len() > 1 || variant.fields.is_empty() {
                panic!("Invalid amount of fields for '{variant_ident}' (required: 1)")
            }

            // todo: had an thought, where we could replace this duplicate code with an enum... look later into plz :3
            quote! {
                #ident::#variant_ident (payload) => {
                    let (ident, event) = (stringify!(#ident), stringify!(#variant_ident));
                    log::trace!("{ident} emitted event [{event}] via provided handle");
                    handle.emit_all(event, payload)
                }
            }
        })
        .collect::<Vec<_>>();

    let impl_enum = quote! {
        impl #ident {
            pub fn with_handle(self, handle: &::tauri::AppHandle) -> Result<(), tauri::Error> {
                use tauri::Manager;

                match self {
                    #( #mapped_variants ),*
                }
            }
        }
    };

    stream.extend(TokenStream::from(impl_enum.to_token_stream()));
    stream
}

#[cfg(feature = "listen")]
#[proc_macro_attribute]
pub fn generate_emit_enum(_: TokenStream, mut stream: TokenStream) -> TokenStream {
    use syn::ItemStruct;

    let cp_stream = stream.clone();
    let item_struct = parse_macro_input!(cp_stream as ItemStruct);

    let ident = format_ident!("{}Emit", item_struct.ident);
    let mapped_variants = item_struct
        .fields
        .iter()
        .map(|field| {
            let field_ty = &field.ty;
            let field_name = field
                .ident
                .as_ref()
                .expect("no type wrapped struct")
                .to_string()
                .to_case(Case::Pascal);
            let field_ident = format_ident!("{field_name}");

            // single enum entry
            quote! { #field_ident (#field_ty) }
        })
        .collect::<Vec<_>>();

    let impl_enum = quote! {
        #[cfg_attr(target_family = "wasm", tauri_interop::listen)]
        #[cfg_attr(not(target_family = "wasm"), tauri_interop::handle_emit_all)]
        #[derive(Debug, Clone)]
        pub enum #ident {
            #( #mapped_variants ),*
        }
    };

    stream.extend(TokenStream::from(impl_enum.to_token_stream()));
    stream
}

#[cfg(feature = "listen")]
#[proc_macro_attribute]
pub fn listen(_: TokenStream, mut stream: TokenStream) -> TokenStream {
    let cp_stream = stream.clone();
    let input_enum = parse_macro_input!(cp_stream as ItemEnum);

    let mapped_variants = input_enum.variants
        .iter()
        .map(|variant| {
            let variant_ident = variant.ident.clone();

            if variant.fields.len() > 1 || variant.fields.is_empty() {
                panic!("Invalid amount of fields for '{variant_ident}' (required: 1)")
            }

            let ty = variant.fields.iter().next().expect("at least one field").ty.clone();
            let fn_ident = variant_ident.to_string().to_case(Case::Snake).to_lowercase();
            let fn_name = format_ident!("listen_to_{fn_ident}");
            // payload struct that is received by 'listen' from tauri,
            // personally i would prefer not to create a struct per enum variation, but we can't serde to JsValue
            let payload_name = format_ident!("Payload{variant_ident}");
            quote! {
                #[derive(Debug, Serialize, Deserialize)]
                struct #payload_name {
                    payload: #ty,
                    event: String
                }

                #[must_use = "if the returned closure isn't preserved, it will never get called, due to being dropped"]
                pub fn #fn_name (callback: impl Fn(#ty) + 'static) -> ::wasm_bindgen::prelude::Closure<dyn Fn(::wasm_bindgen::JsValue)> {
                    let conversion_closure = ::wasm_bindgen::prelude::Closure::new(move |value| {
                        let payload = serde_wasm_bindgen::from_value::<#payload_name>(value).expect("serializable");
                        callback(payload.payload)
                    });

                    crate::listen(stringify!(#variant_ident), &conversion_closure);

                    conversion_closure
                }
            }
        }).collect::<Vec<_>>();

    stream.extend(TokenStream::from(
        quote!( #( #mapped_variants )* ).to_token_stream(),
    ));
    stream
}

lazy_static::lazy_static! {
    static ref HANDLER_LIST: Mutex<BTreeSet<String>> = Mutex::new(Default::default());
}

const TAURI_TYPES: [&str; 3] = [ "State", "AppHandle", "Window" ];

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
    let ItemFn {
        attrs,
        sig,
        ..
    } = syn::parse::<ItemFn>(stream).unwrap();

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
                }
            )
        },
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
                    return Some(fn_inputs)
                }
            }
            None
        })
        .collect::<Punctuated<FnArg, Comma>>();

    let args_ident = format_ident!("{}Args", ident.to_string().to_case(Case::Pascal));

    let invoke = if need_catch {
        quote! {
            self::tauri_interop_invoke_catch(stringify!(#ident), args).await
                .map(|value| serde_wasm_bindgen::from_value(value).expect("ok: conversion error"))
                .map_err(|value| serde_wasm_bindgen::from_value(value).expect("err: conversion error"))
        }
    } else if async_ident.is_some() {
        quote! { 
            let value = self::tauri_interop_invoke_promise(stringify!(#ident), args).await;
            serde_wasm_bindgen::from_value(value).expect("conversion error")
        }
    } else {
        quote! { 
            self::tauri_interop_invoke(stringify!(#ident), args);
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

    HANDLER_LIST.lock().unwrap().insert(fn_item.sig.ident.to_string());

    let command_macro = quote! {
        #[cfg_attr(target_family = "wasm", tauri_interop::command)]
        #[cfg_attr(not(target_family = "wasm"), tauri::command)]
        #fn_item
    };

    TokenStream::from(command_macro.to_token_stream())
}

#[proc_macro]
pub fn setup(_: TokenStream) -> TokenStream {
    let handler = HANDLER_LIST.lock().unwrap();
    let handler = handler
        .iter()
        .map(|s| format_ident!("{s}"))
        .collect::<Punctuated<Ident, Comma>>();

    let stream = quote! {
        #[cfg(target_family = "wasm")]
        #[::wasm_bindgen::prelude::wasm_bindgen]
        extern "C" {
            #[::wasm_bindgen::prelude::wasm_bindgen(js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
            fn tauri_interop_invoke(cmd: &str, args: ::wasm_bindgen::JsValue);

            #[::wasm_bindgen::prelude::wasm_bindgen(js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
            async fn tauri_interop_invoke_promise(cmd: &str, args: ::wasm_bindgen::JsValue) -> ::wasm_bindgen::JsValue;

            #[::wasm_bindgen::prelude::wasm_bindgen(catch, js_name = "invoke", js_namespace = ["window", "__TAURI__", "tauri"])]
            async fn tauri_interop_invoke_catch(cmd: &str, args: ::wasm_bindgen::JsValue) -> Result<::wasm_bindgen::JsValue, ::wasm_bindgen::JsValue>;
        }

        #[cfg(not(target_family = "wasm"))]
        /// the all mighty collector
        pub fn get_handlers() -> impl Fn(tauri::Invoke) {
            tauri::generate_handler![ #handler ]
        }
    };

    TokenStream::from(stream.to_token_stream())
}

#[proc_macro_attribute]
pub fn conditional_use(_: TokenStream, stream: TokenStream) -> TokenStream {
    let item_use = syn::parse::<ItemUse>(stream).unwrap();

    let command_macro = quote! {
        #[cfg(not(target_family = "wasm"))]
        #item_use
    };

    TokenStream::from(command_macro.to_token_stream())
}
