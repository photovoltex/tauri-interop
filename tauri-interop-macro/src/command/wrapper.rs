use proc_macro::Span;

use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{
    parse_quote, Attribute, Expr, FnArg, GenericParam, Generics, ItemFn, Lifetime, LifetimeParam,
    Pat, ReturnType, Signature, Type, TypePath,
};

#[derive(PartialEq)]
pub enum Invoke {
    Empty,
    AsyncEmpty,
    Async,
    AsyncResult,
}

impl Invoke {
    pub fn as_async(&self) -> Option<Ident> {
        self.ne(&Invoke::Empty).then_some(format_ident!("async"))
    }

    pub fn as_expr(&self, cmd_name: String, arg_name: &Ident) -> Expr {
        let expr: Ident = match self {
            Invoke::Empty => parse_quote!(fire_and_forget_invoke),
            Invoke::AsyncEmpty => parse_quote!(wait_invoke),
            Invoke::Async => parse_quote!(return_invoke),
            Invoke::AsyncResult => parse_quote!(catch_invoke),
        };

        let call = parse_quote!( ::tauri_interop::command::bindings::#expr(#cmd_name, #arg_name) );

        if self.as_async().is_some() {
            Expr::Await(parse_quote!(#call.await))
        } else {
            Expr::Call(call)
        }
    }
}

fn is_result(type_path: &TypePath) -> bool {
    type_path
        .path
        .segments
        .iter()
        .any(|segment| "Result".eq(&segment.ident.to_string()))
}

fn determine_invoke(return_type: &ReturnType, is_async: bool) -> Invoke {
    match return_type {
        ReturnType::Default if is_async => Invoke::AsyncEmpty,
        ReturnType::Default => Invoke::Empty,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(path) if is_result(path) => Invoke::AsyncResult,
            Type::Path(_) => Invoke::Async,
            others => panic!("no support for '{}'", others.to_token_stream()),
        },
    }
}

const ARGUMENT_LIFETIME: &str = "'arg_lifetime";

fn new_arg_lt() -> Lifetime {
    Lifetime::new(ARGUMENT_LIFETIME, Span::call_site().into())
}

fn any_tauri(ty_path: &TypePath) -> bool {
    ty_path
        .path
        .segments
        .iter()
        .any(|segment| segment.ident.to_string().to_lowercase().contains("tauri"))
}

pub struct InvokeCommand {
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    pub generics: Generics,
    pub return_type: ReturnType,
    pub invoke: Invoke,
    pub invoke_argument: InvokeArgument,
}

pub struct InvokeArgument {
    pub argument_name: Ident,
    pub fields: Vec<FieldArg>,
}

pub struct FieldArg {
    pub ident: Ident,
    pub argument: FnArg,
    requires_lifetime: bool,
}

pub fn prepare(function: ItemFn) -> InvokeCommand {
    let ItemFn {
        attrs: attributes,
        sig,
        ..
    } = function;

    let Signature {
        ident: name,
        mut generics,
        inputs,
        output: return_type,
        asyncness,
        ..
    } = sig;

    let filtered_fields = inputs
        .into_iter()
        .filter_map(|mut fn_arg| {
            let typed = match fn_arg {
                FnArg::Typed(ref mut typed) => typed,
                _ => return None,
            };

            if matches!(typed.ty.as_ref(), Type::Path(ty_path) if any_tauri(ty_path)) {
                return None;
            }

            let req_lf = if let Type::Reference(ty_ref) = typed.ty.as_mut() {
                ty_ref.lifetime = Some(new_arg_lt());
                true
            } else {
                false
            };

            match typed.pat.as_mut() {
                Pat::Ident(ident) => {
                    // converting the ident to snake case, so it matches the expected snake case
                    ident.ident = format_ident!("{}", ident.ident.to_string().to_case(Case::Snake));
                    Some(FieldArg {
                        ident: ident.ident.clone(),
                        argument: fn_arg,
                        requires_lifetime: req_lf,
                    })
                },
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    if filtered_fields.iter().any(|field| field.requires_lifetime) {
        generics
            .params
            .push(GenericParam::Lifetime(LifetimeParam::new(new_arg_lt())))
    }

    let invoke = determine_invoke(&return_type, asyncness.is_some());
    let argument_name = format_ident!("{}Args", name.to_string().to_case(Case::Pascal));

    InvokeCommand {
        attributes,
        name,
        generics,
        return_type,
        invoke,
        invoke_argument: InvokeArgument {
            argument_name,
            fields: filtered_fields,
        },
    }
}
