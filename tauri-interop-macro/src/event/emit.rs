use crate::event::{Event, EventField, Field, FieldAttributes};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, ItemStruct};

pub fn derive(stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as ItemStruct);
    let Event {
        ident,
        mod_name,
        fields,
    } = super::prepare(stream_struct);

    let emit_fields = fields.iter().map(|field| {
        let EventField { name, field, ty } = field;

        quote! {
            #[allow(dead_code)]
            #[derive(::tauri_interop::EmitField)]
            #[parent(#ident)]
            #[field_name(#field)]
            #[field_ty(#ty)]
            pub struct #name;
        }
    });

    let event_fields = fields.iter().map(|field| &field.name);

    let stream = quote! {
        use tauri_interop::event::{Field, emit::Emit};

        pub mod #mod_name {
            use super::#ident;
            use tauri_interop::event::{Field, emit::Emit};

            // to each field a defined struct tuple is generated
            #( #emit_fields )*
        }

        impl Emit for #ident {
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
        ident,
        attributes,
        event,
    } = super::prepare_field(derive_input);

    let FieldAttributes { parent, name, ty } = attributes;

    name.as_ref().expect("name attribute was expected");

    let stream = quote! {
        impl Field<#parent> for #ident {
            type Type = #ty;

            fn emit(parent: &#parent, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error> {
                use ::tauri::Manager;

                log::trace!("Emitted event [{}]", #event);

                handle.emit_all(#event, parent.#name.clone())
            }

            fn update(parent: &mut #parent, handle: &::tauri::AppHandle, v: Self::Type) -> Result<(), ::tauri::Error> {
                parent.#name = v;
                Self::emit(parent, handle)
            }
        }
    };

    TokenStream::from(stream.to_token_stream())
}
