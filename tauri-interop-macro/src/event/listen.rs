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
            use super::#name;
            use ::tauri_interop::event::Field;

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
    } = super::prepare_field(derive_input);

    let FieldAttributes {
        parent,
        parent_field_ty,
        ..
    } = attributes;

    let stream = quote! {
        impl Field<#parent> for #name {
            type Type = #parent_field_ty;
            const EVENT_NAME: &'static str = #event_name;
        }
    };

    TokenStream::from(stream.to_token_stream())
}
