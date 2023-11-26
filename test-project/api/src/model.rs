#[allow(dead_code)]
#[derive(Default)]
#[tauri_interop::emit_or_listen]
pub struct TestState {
    foo: String,
    pub bar: bool,
}

// /// not allowed
// #[tauri_interop::emit_or_listen]
// pub struct StructTupleState(String);

// /// not allowed
// #[tauri_interop::emit_or_listen]
// pub struct PanicState {}
