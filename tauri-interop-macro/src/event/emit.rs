use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{DeriveInput, parse_macro_input};

use crate::event::{EventField, EventStruct, Field, FieldAttributes};

pub fn derive(stream: TokenStream) -> TokenStream {
    let stream_struct = parse_macro_input!(stream as DeriveInput);
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
    let commands_attr = cfg!(feature = "initial_value").then_some(quote!(#[::tauri_interop::commands])).unwrap_or_default();
    let collect_command = cfg!(feature = "initial_value").then_some(quote!(::tauri_interop::collect_commands!();)).unwrap_or_default();

    let stream = quote! {
        #commands_attr
        pub mod #mod_name {
            use super::#name;

            #( #emit_fields )*
            
            #collect_command
        }

        impl ::tauri_interop::event::Emit for #name {
            fn emit_all(&self, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error> {
                use #mod_name::*;
                use ::tauri_interop::event::Field;

                #( #event_fields::emit(self, handle)?; )*

                Ok(())
            }

            fn emit<F: ::tauri_interop::event::Field<Self>>(&self, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error>
            where
                Self: Sized
            {
                use ::tauri_interop::event::Field;
                F::emit(self, handle)
            }

            fn update<F: ::tauri_interop::event::Field<Self>>(&mut self, handle: &::tauri::AppHandle, field: F::Type) -> Result<(), ::tauri::Error>
            where
                Self: Sized
            {
                use ::tauri_interop::event::Field;
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
        get_cmd
    } = super::prepare_field(derive_input);

    let FieldAttributes {
        parent,
        parent_field_name,
        parent_field_ty,
    } = attributes;

    parent_field_name
        .as_ref()
        .expect("name attribute was expected");

    // todo: currently we only resolve the parent type, if the parent type is wrapped to allow inner mutability we can't acquire the state 
    let get_cmd = cfg!(feature = "initial_value").then_some(quote! {
            #[allow(non_snake_case)]
            #[tauri_interop::command]
            pub fn #get_cmd(handle: ::tauri::AppHandle) -> Result<#parent_field_ty, ::tauri_interop::event::EventError> {
                use ::tauri::Manager;
                use ::tauri_interop::event::Field;
                
                let state = handle.try_state::<#parent>()
                    .ok_or(::tauri_interop::event::EventError::StateIsNotRegistered(stringify!(#parent).into()))?;
                Ok(#name::get_value(&state))
            }
        }).unwrap_or_default();
    
    let get_value = cfg!(feature = "initial_value").then_some(quote!{
        fn get_value(parent: &#parent) -> Self::Type {
            parent.#parent_field_name.clone()
        }
    }).unwrap_or_default();

    let stream = quote! {
        impl ::tauri_interop::event::Field<#parent> for #name {
            type Type = #parent_field_ty;

            const EVENT_NAME: &'static str = #event_name;

            #get_value

            fn emit(parent: &#parent, handle: &::tauri::AppHandle) -> Result<(), ::tauri::Error> {
                use ::tauri::Manager;

                log::trace!("Emitted event [{}]", #event_name);

                handle.emit_all(#event_name, Self::get_value(parent))
            }

            fn update(parent: &mut #parent, handle: &::tauri::AppHandle, v: Self::Type) -> Result<(), ::tauri::Error> {
                parent.#parent_field_name = v;
                Self::emit(parent, handle)
            }
        }
        
        #get_cmd
    };

    TokenStream::from(stream.to_token_stream())
}
