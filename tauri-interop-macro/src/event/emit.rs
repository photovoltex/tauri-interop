use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, ItemStruct};

use crate::event::{EventField, EventStruct, Field, FieldAttributes};

pub fn derive(stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);
    let EventStruct {
        name,
        mod_name,
        fields,
    } = super::prepare_event(stream_struct);

    let emit_fields = fields.iter().map(|field| {
        let EventField {
            field_name,
            parent_field_name,
            parent_field_ty,
        } = field;

        quote! {
            #[allow(dead_code)]
            #[derive(::tauri_interop::EmitField)]
            #[parent(#name)]
            #[parent_field_name(#parent_field_name)]
            #[parent_field_ty(#parent_field_ty)]
            pub struct #field_name;
        }
    });

    let event_fields = fields.iter().map(|field| &field.field_name);

    let stream = quote! {
        use tauri_interop::event::{Field, emit::Emit};

        pub mod #mod_name {
            use super::#name;
            use tauri_interop::event::{Field, emit::Emit};

            #( #emit_fields )*
        }

        impl Emit for #name {
            fn emit_all(&self, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error> {
                use #mod_name::*;

                #( #event_fields::emit(self, handle)?; )*

                Ok(())
            }

            fn emit<F: Field<Self>>(&self, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error>
            where
                Self: Sized
            {
                F::emit(self, handle)
            }

            fn update<F: Field<Self>>(&mut self, handle: &::tauri::AppHandle, field: F::Type) -> Result<(), ::tauri::Error>
            where
                Self: Sized
            {
                F::update(self, handle, field)
            }
        }
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
        parent_field_name,
        parent_field_ty,
    } = attributes;

    parent_field_name
        .as_ref()
        .expect("name attribute was expected");

    let stream = quote! {
        impl Field<#parent> for #name {
            type Type = #parent_field_ty;

            fn emit(parent: &#parent, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error> {
                use ::tauri::Manager;

                log::trace!("Emitted event [{}]", #event_name);

                handle.emit_all(#event_name, parent.#parent_field_name.clone())
            }

            fn update(parent: &mut #parent, handle: &::tauri::AppHandle, v: Self::Type) -> Result<(), ::tauri::Error> {
                parent.#parent_field_name = v;
                Self::emit(parent, handle)
            }
        }
    };

    TokenStream::from(stream.to_token_stream())
}
