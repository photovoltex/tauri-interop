use std::fmt::Display;
use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::format_ident;
use syn::{ItemStruct, Type, Attribute, DeriveInput};

pub(crate) mod emit;
pub(crate) mod listen;

struct Event {
    ident: Ident,
    mod_name: Ident,
    fields: Vec<EventField>,
}

struct EventField {
    name: Ident,
    field: Ident,
    ty: Type,
}

fn prepare(stream_struct: ItemStruct) -> Event {
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

    let struct_name = &stream_struct.ident;
    let mod_name = struct_name.to_string().to_case(Case::Snake);

    let fields = stream_struct
        .fields
        .iter()
        .map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let field_name = field_ident.to_string().to_case(Case::Pascal);

            EventField {
                name: field_ident.clone(),
                field: format_ident!("{field_name}"),
                ty: field.ty.clone(),
            }
        })
        .collect::<Vec<_>>();

    Event {
        ident: struct_name.clone(),
        mod_name: format_ident!("{mod_name}"),
        fields,
    }
}

struct Field {
    ident: Ident,
    attributes: FieldAttributes,
    event: String,
}

struct FieldAttributes {
    pub parent: Ident,
    pub name: Option<Ident>,
    pub ty: Type,
}

fn get_field_values(attrs: Vec<Attribute>) -> FieldAttributes {
    let parent = attrs
        .iter()
        .find(|a| a.path().is_ident("parent"))
        .expect("expected parent attribute")
        .parse_args()
        .unwrap();

    let name = attrs
        .iter()
        .find(|a| a.path().is_ident("field_name"))
        .and_then(|name| name.parse_args().ok());

    let ty = attrs
        .iter()
        .find(|a| a.path().is_ident("field_ty"))
        .expect("expected ty attribute")
        .parse_args()
        .unwrap();

    FieldAttributes { parent, name, ty }
}

/// function to build the same unique event name for wasm and host triplet
fn get_event_name<S, F>(struct_name: &S, field_name: &F) -> String
    where
        S: Display,
        F: Display,
{
    format!("{struct_name}::{field_name}")
}

fn prepare_field(derive_input: DeriveInput) -> Field {
    let ident = derive_input.ident.clone();
    let attributes = get_field_values(derive_input.attrs);

    Field{
        event: get_event_name(&attributes.parent, &ident),
        ident,
        attributes,
    }
}
