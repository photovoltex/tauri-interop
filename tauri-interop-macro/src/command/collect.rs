use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_quote, ExprPath};

pub fn commands_with_mod_name(mod_name: &str, commands: &HashSet<String>) -> HashSet<String> {
    commands
        .iter()
        .map(|cmd| format!("{mod_name}::{cmd}"))
        .collect()
}

pub fn commands_to_punctuated(commands: &HashSet<String>) -> Punctuated<ExprPath, Comma> {
    commands.iter().map(command_to_expr_path).collect()
}

pub fn command_to_expr_path(command: &String) -> ExprPath {
    match get_separated_command(command) {
        None => {
            let ident = format_ident!("{command}");
            parse_quote!(#ident)
        }
        Some((mod_name, cmd_name)) => parse_quote!(#mod_name::#cmd_name),
    }
}

pub fn get_separated_command(input: &str) -> Option<(Ident, Ident)> {
    let mut split_cmd = input.split("::");
    let mod_name = format_ident!("{}", split_cmd.next()?);
    // order matters
    let cmd_name = format_ident!("{}", split_cmd.next()?);

    Some((mod_name, cmd_name))
}

pub fn get_handler_function(
    fn_name: Ident,
    commands: &HashSet<String>,
    handlers: Punctuated<ExprPath, Comma>,
    include_mods: Vec<ExprPath>,
) -> TokenStream {
    let commands = commands.iter().collect::<Vec<_>>();
    quote! {
        #[cfg(not(target_family = "wasm"))]
        #[doc = "auto generated function to register all configured commands"]
        pub fn #fn_name() -> impl Fn(tauri::Invoke) {
            #( use #include_mods; )*

            let handlers = vec! [ #( #commands ),* ];
            log::debug!("Registering following commands to tauri: {handlers:#?}");

            ::tauri::generate_handler![ #handlers ]
        }
    }
}
