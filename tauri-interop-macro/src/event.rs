use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::format_ident;
use syn::{Attribute, Data, DeriveInput, Type};

pub(crate) mod emit;
pub(crate) mod listen;

struct EventStruct {
    name: Ident,
    mod_name: Ident,
    fields: Vec<EventField>,
}

struct EventField {
    field_name: Ident,
    parent_field_name: Ident,
    parent_field_ty: Type,
}

fn prepare_event(derive_input: DeriveInput) -> EventStruct {
    let data_struct = match derive_input.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("The macro only works with structs"),
    };

    if data_struct.fields.is_empty() {
        panic!("No fields provided")
    }

    if data_struct.fields.iter().any(|field| field.ident.is_none()) {
        panic!("Tuple Structs aren't supported")
    }

    let auto_naming = derive_input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("auto_naming"))
        .map(|attr| attr.parse_args::<Ident>().unwrap().to_string());

    let name = derive_input.ident.clone();
    let (naming_case, mod_name) = match auto_naming {
        Some(naming) if naming == "EnumLike" => (Case::Pascal, format!("{name}Field")),
        Some(naming) => panic!("No naming type found for: {naming}"),
        None => (Case::Snake, name.to_string()),
    };

    let mod_name = derive_input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("mod_name"))
        .map(|attr| attr.parse_args::<Ident>().unwrap())
        .unwrap_or(format_ident!("{}", mod_name.to_case(naming_case)));

    let fields = data_struct
        .fields
        .iter()
        .map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let field_name = format_ident!("{}", field_ident.to_string().to_case(Case::Pascal));

            EventField {
                field_name,
                parent_field_name: field_ident.clone(),
                parent_field_ty: field.ty.clone(),
            }
        })
        .collect::<Vec<_>>();

    EventStruct {
        name,
        mod_name,
        fields,
    }
}

struct Field {
    name: Ident,
    attributes: FieldAttributes,
    event_name: String,
}

struct FieldAttributes {
    pub parent: Ident,
    pub parent_field_name: Option<Ident>,
    pub parent_field_ty: Type,
}

fn get_field_values(attrs: Vec<Attribute>) -> FieldAttributes {
    let parent = attrs
        .iter()
        .find(|a| a.path().is_ident("parent"))
        .expect("expected parent attribute")
        .parse_args()
        .unwrap();

    let parent_field_name = attrs
        .iter()
        .find(|a| a.path().is_ident("parent_field_name"))
        .and_then(|name| name.parse_args().ok());

    let parent_field_ty = attrs
        .iter()
        .find(|a| a.path().is_ident("parent_field_ty"))
        .expect("expected ty attribute")
        .parse_args()
        .unwrap();

    FieldAttributes {
        parent,
        parent_field_name,
        parent_field_ty,
    }
}

fn prepare_field(derive_input: DeriveInput) -> Field {
    let name = derive_input.ident.clone();
    let attributes = get_field_values(derive_input.attrs);
    let event_name = format!("{}::{}", &attributes.parent, &name);

    Field {
        event_name,
        name,
        attributes,
    }
}
