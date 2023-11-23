#[tauri_interop::emit_or_listen]
pub struct TestState {
    pub echo: String,
    pub foo: i32,
    pub bar: bool,
}
