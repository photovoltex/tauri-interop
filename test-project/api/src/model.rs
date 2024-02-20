use tauri_interop::Event;

#[allow(dead_code)]
#[derive(Default, Event)]
pub struct TestState {
    foo: String,
    pub bar: bool,
}

// /// not allowed
// #[derive(Default, Event)]
// pub struct StructTupleState(String);

// /// not allowed
// #[derive(Default, Event)]
// pub struct PanicState {}
