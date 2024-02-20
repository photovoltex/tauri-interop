use crate::event::{Event, EventField, Field, FieldAttributes};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemStruct, DeriveInput};

pub fn derive(stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);
    let Event {
        ident,
        mod_name,
        fields,
    } = super::prepare(stream_struct);

    let listen_fields = fields.iter().map(|field| {
        let EventField { name, ty, .. } = field;

        quote! {
            #[allow(dead_code)]
            #[derive(::tauri_interop::ListenField)]
            #[parent(#ident)]
            #[field_ty(#ty)]
            pub struct #name;
        }
    });

    let stream = quote! {
        pub mod #mod_name {
            use super::#ident;
            use ::tauri_interop::event::Field;

            #( #listen_fields )*
        }

        impl ::tauri_interop::event::listen::Listen for #ident {}
    };

    TokenStream::from(stream.to_token_stream())
}

pub fn derive_field(stream: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(stream as DeriveInput);

    let Field {
        ident,
        attributes,
        event,
    } = super::prepare_field(derive_input);

    let FieldAttributes { parent, ty, .. } = attributes;

    let stream = quote! {
        impl Field<#parent> for #ident {
            type Type = #ty;
            const EVENT_NAME: &'static str = #event;
        }
    };

    TokenStream::from(stream.to_token_stream())
}
