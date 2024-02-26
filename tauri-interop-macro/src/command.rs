use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, FnArg, ItemFn};

use crate::command::wrapper::{InvokeArgument, InvokeCommand};

mod wrapper;

pub fn convert_to_binding(stream: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(stream as ItemFn);
    let InvokeCommand {
        attributes,
        name,
        generics,
        return_type,
        invoke,
        invoke_argument,
    } = wrapper::prepare(item_fn);

    let InvokeArgument {
        argument_name,
        fields,
    } = invoke_argument;

    let async_ident = invoke.as_async();
    let field_usage = fields
        .iter()
        .map(|field| field.ident.clone())
        .collect::<Punctuated<Ident, Comma>>();
    let field_definitions = fields
        .into_iter()
        .map(|field| field.argument)
        .collect::<Punctuated<FnArg, Comma>>();

    let command_name = name.to_string();
    let args_ident = format_ident!("args");
    let invoke_binding = invoke.as_expr(command_name, &args_ident);

    let stream = quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        struct #argument_name #generics {
            #field_definitions
        }

        #( #attributes )*
        pub #async_ident fn #name #generics (#field_definitions) #return_type
        {
            let #args_ident = #argument_name { #field_usage };
            let #args_ident = ::serde_wasm_bindgen::to_value(&#args_ident)
                .expect("serialized arguments");

            #invoke_binding
        }
    };

    TokenStream::from(stream.to_token_stream())
}
