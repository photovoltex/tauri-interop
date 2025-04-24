#![allow(clippy::no_effect)]
#![allow(dead_code)]
#![allow(path_statements)]

// this mod at this position doesn't make much sense logic vise
// for testing the combine feature tho it's a quite convenient spot :D
#[cfg(not(target_family = "wasm"))]
mod host_impl;
#[tauri_interop::commands]
pub mod other_cmd;

use tauri_interop::{Event, ManagedEmit};

#[derive(Default, Event)]
#[mod_name(test_mod)]
pub struct TestState {
    foo: String,
    pub bar: bool,
}

#[derive(Default, Event, ManagedEmit)]
#[auto_naming(EnumLike)]
pub struct NamingTestEnum {
    foo: String,
    pub bar: bool,
}

#[derive(Default, Event, ManagedEmit)]
pub struct NamingTestDefault {
    foo: String,
    pub bar: bool,
}

fn test_naming() {
    test_mod::FBar;
    test_mod::FFoo;
    NamingTestEnumField::FBar;
    NamingTestEnumField::FFoo;
    naming_test_default::FBar;
    naming_test_default::FFoo;
}

// /// not allowed
// #[derive(Default, Event)]
// pub struct StructTupleState(String);

// /// not allowed
// #[derive(Default, Event)]
// pub struct PanicState {}
