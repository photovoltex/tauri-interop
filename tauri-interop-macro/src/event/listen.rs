use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

use crate::event::{EventField, EventStruct, Field, FieldAttributes};

pub fn derive(stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as DeriveInput);
    let EventStruct {
        name,
        mod_name,
        fields,
    } = super::prepare_event(stream_struct);

    let listen_fields = fields.iter().map(|field| {
        let EventField {
            field_name,
            parent_field_ty,
            ..
        } = field;

        quote! {
            #[allow(dead_code)]
            #[derive(::tauri_interop::ListenField)]
            #[parent(#name)]
            #[parent_field_ty(#parent_field_ty)]
            pub struct #field_name;
        }
    });

    let stream = quote! {
        pub mod #mod_name {
            use super::*;

            #( #listen_fields )*
        }

        impl ::tauri_interop::event::Listen for #name {}
    };

    TokenStream::from(stream.to_token_stream())
}

pub fn derive_field(stream: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(stream as DeriveInput);

    let Field {
        name,
        attributes,
        event_name,
        get_cmd,
    } = super::prepare_field(derive_input);

    let FieldAttributes {
        parent,
        parent_field_ty,
        ..
    } = attributes;

    let get_cmd_fn = cfg!(feature = "initial_value")
        .then_some(quote! {
            #[allow(non_snake_case)]
            #[tauri_interop::command]
            pub fn #get_cmd() -> Result<#parent_field_ty, ::tauri_interop::event::EventError> {}
        })
        .unwrap_or_default();

    let get_value = cfg!(feature = "initial_value")
        .then_some(quote! {
            async fn get_value() -> Result<Self::Type, ::tauri_interop::event::EventError> {
                #get_cmd().await
            }
        })
        .unwrap_or_default();

    let stream = quote! {
        #get_cmd_fn

        impl ::tauri_interop::event::Field<#parent> for #name {
            type Type = #parent_field_ty;
            const EVENT_NAME: &'static str = #event_name;

            #get_value
        }
    };

    TokenStream::from(stream.to_token_stream())
}
