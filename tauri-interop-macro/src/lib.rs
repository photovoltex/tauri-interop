#![warn(missing_docs)]
//! The macros use by `tauri_interop` to generate dynamic code depending on the target

#[cfg(feature = "event")]
use std::fmt::Display;
use std::{collections::BTreeSet, sync::Mutex};

use convert_case::{Case, Casing};
use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, DeriveInput, FnArg, Ident,
    ItemFn, ItemUse, Lifetime, LifetimeParam, Pat, PathSegment, ReturnType, Signature, Type,
};

#[cfg(feature = "event")]
use syn::ItemStruct;

/// Conditionally adds [macro@listen_to] or [Emit] to a struct
#[cfg(feature = "event")]
#[proc_macro_attribute]
pub fn emit_or_listen(_: TokenStream, stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);
    let stream = quote! {
        #[cfg_attr(target_family = "wasm", derive(tauri_interop::Listen))]
        #[cfg_attr(not(target_family = "wasm"), derive(tauri_interop::Emit))]
        #stream_struct
    };

    TokenStream::from(stream.to_token_stream())
}

/// function to build the same unique event name for wasm and host triplet
#[cfg(feature = "event")]
fn get_event_name<S, F>(struct_name: &S, field_name: &F) -> String
where
    S: Display,
    F: Display,
{
    format!("{struct_name}::{field_name}")
}

/// Generates an default `Emit` implementation for the given struct with a
/// correlation enum, mod and field-structs for emitting a single field of
/// the struct.
///
/// Used for host code generation.
#[cfg(feature = "event")]
#[proc_macro_derive(Emit)]
pub fn derive_emit(stream: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(stream as ItemStruct);

    if item_struct.fields.is_empty() {
        panic!("No fields provided")
    }

    if item_struct.fields.iter().any(|field| field.ident.is_none()) {
        panic!("Tuple Structs aren't supported")
    }

    let struct_name = &item_struct.ident;

    let mod_name = struct_name.to_string().to_case(Case::Snake);
    let mod_name = format_ident!("{mod_name}");

    let mut fields = vec![];
    let struct_field_fields = item_struct
        .fields
        .iter()
        .map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let field_name = field_ident.to_string().to_case(Case::Pascal);

            let field_name = format_ident!("{field_name}");
            let field_ty = &field.ty;

            fields.push(field_name.clone());

            quote! {
                #[allow(dead_code)]
                #[derive(::tauri_interop::EmitField)]
                #[parent(#struct_name)]
                #[name(#field_ident)]
                #[ty(#field_ty)]
                pub struct #field_name;
            }
        })
        .collect::<Vec<_>>();

    let stream = quote! {
        use tauri_interop::event::emit::{Emit, EmitField};

        pub mod #mod_name {
            use super::#struct_name;
            use tauri_interop::event::emit::{Emit, EmitField};

            // to each field a defined struct tuple is provided
            #( #struct_field_fields )*
        }

        impl Emit for #struct_name {
            fn emit_all(&self, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error> {
                use #mod_name::*;

                #( #fields::emit(self, handle)?; )*

                Ok(())
            }

            fn emit<F: EmitField<Self>>(&self, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error>
            where
                Self: Sized
            {
                F::emit(self, handle)
            }

            fn update<F: EmitField<Self>>(&mut self, handle: &::tauri::AppHandle, field: F::Type) -> Result<(), ::tauri::Error>
            where
                Self: Sized
            {
                F::update(self, handle, field)
            }
        }
    };

    TokenStream::from(stream.to_token_stream())
}

struct FieldValues {
    pub parent: Ident,
    pub name: Option<Ident>,
    pub ty: Type,
}

fn get_field_values(attrs: Vec<Attribute>) -> FieldValues {
    let parent = attrs
        .iter()
        .find(|a| a.path().is_ident("parent"))
        .expect("expected parent attribute")
        .parse_args()
        .unwrap();

    let name = attrs
        .iter()
        .find(|a| a.path().is_ident("name"))
        .and_then(|name| name.parse_args().ok());

    let ty = attrs
        .iter()
        .find(|a| a.path().is_ident("ty"))
        .expect("expected ty attribute")
        .parse_args()
        .unwrap();

    FieldValues { parent, name, ty }
}

/// Generates an default `EmitField` implementation for the given struct.
///
/// Used for host code generation.
#[cfg(feature = "event")]
#[proc_macro_derive(EmitField, attributes(parent, name, ty))]
pub fn derive_emit_field(stream: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(stream as DeriveInput);

    let FieldValues { parent, name, ty } = get_field_values(derive_input.attrs);
    let name = name.expect("name attribute was expected");
    let struct_name = &derive_input.ident;

    let event_name = get_event_name(&parent, struct_name);

    let stream = quote! {
        impl EmitField<#parent> for #struct_name {
            type Type = #ty;

            fn emit(parent: &#parent, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error> {
                use tauri::Manager;

                log::trace!(
                    "Emitted event [{}::{}]",
                    stringify!(#parent),
                    stringify!(#struct_name),
                );

                handle.emit_all(#event_name, parent.#name.clone())
            }

            fn update(parent: &mut #parent, handle: &::tauri::AppHandle, v: Self::Type) -> Result<(), ::tauri::Error> {
                parent.#name = v;
                Self::emit(parent, handle)
            }
        }
    };

    TokenStream::from(stream.to_token_stream())
}

/// Generates `listen_to_<field>` functions for the given
/// struct for the correlating host code.
///
/// Used for wasm code generation
#[cfg(feature = "event")]
#[proc_macro_derive(Listen)]
pub fn derive_listen(stream: TokenStream) -> TokenStream {
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

    let mod_name = struct_ident.to_string().to_case(Case::Snake);
    let mod_name = format_ident!("{mod_name}");

    let struct_field_fields = stream_struct
        .fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            let field_ident = field
                .ident
                .as_ref()
                .expect("handled before")
                .to_string()
                .to_case(Case::Pascal);
            let field_ident = format_ident!("{field_ident}");

            quote! {
                #[derive(::tauri_interop::ListenField)]
                #[parent(#struct_ident)]
                #[ty(#ty)]
                pub struct #field_ident;
            }
        })
        .collect::<Vec<_>>();

    let stream = quote! {
        pub mod #mod_name {
            use super::#struct_ident;
            use ::tauri_interop::event::listen::ListenField;

            #( #struct_field_fields )*
        }

        impl ::tauri_interop::event::listen::Listen for #struct_ident {}
    };

    TokenStream::from(stream.to_token_stream())
}

/// Generates an default `ListenField` implementation for the given struct.
///
/// Used for wasm code generation.
#[cfg(feature = "event")]
#[proc_macro_derive(ListenField, attributes(parent, ty))]
pub fn derive_listen_field(stream: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(stream as DeriveInput);

    let FieldValues { parent, ty, .. } = get_field_values(derive_input.attrs);
    let struct_name = &derive_input.ident;

    let event_name = get_event_name(&parent, &struct_name);

    let stream = quote! {
        impl ListenField<#parent> for #struct_name {
            type Type = #ty;
            const EVENT_NAME: &'static str = #event_name;
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

#[derive(PartialEq)]
enum Invoke {
    Empty,
    AsyncEmpty,
    Async,
    AsyncResult,
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
        asyncness,
        ..
    } = sig;

    let invoke_type = match &output {
        ReturnType::Default => {
            if asyncness.is_some() {
                Invoke::AsyncEmpty
            } else {
                Invoke::Empty
            }
        }
        ReturnType::Type(_, ty) => match ty.as_ref() {
            // fixme: if it's an single ident, catch isn't needed this could probably be a problem later
            Type::Path(path) if path.path.segments.iter().any(is_result) => Invoke::AsyncResult,
            Type::Path(_) => Invoke::Async,
            others => panic!("no support for '{}'", others.to_token_stream()),
        },
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

    let async_ident = invoke_type.ne(&Invoke::Empty).then_some(format_ident!("async"));
    let invoke = match invoke_type {
        Invoke::Empty => quote!(::tauri_interop::bindings::invoke(stringify!(#ident), args);),
        Invoke::Async | Invoke::AsyncEmpty => {
            quote!(::tauri_interop::command::async_invoke(stringify!(#ident), args).await)
        }
        Invoke::AsyncResult => {
            quote!(::tauri_interop::command::invoke_catch(stringify!(#ident), args).await)
        }
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
