#![allow(clippy::no_effect)]
#![allow(dead_code)]
#![allow(path_statements)]

use tauri_interop::Event;

#[derive(Default, Event)]
#[mod_name(test_mod)]
pub struct TestState {
    foo: String,
    pub bar: bool,
}

#[derive(Default, Event)]
#[auto_naming(EnumLike)]
pub struct NamingTestEnum {
    foo: String,
    pub bar: bool,
}

#[derive(Default, Event)]
pub struct NamingTestDefault {
    foo: String,
    pub bar: bool,
}

fn test_naming() {
    test_mod::Bar;
    test_mod::Foo;
    NamingTestEnumField::Bar;
    NamingTestEnumField::Foo;
    naming_test_default::Bar;
    naming_test_default::Foo;
}

// /// not allowed
// #[derive(Default, Event)]
// pub struct StructTupleState(String);

// /// not allowed
// #[derive(Default, Event)]
// pub struct PanicState {}
