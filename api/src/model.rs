
#[tauri_interop::conditional_emit]
pub struct TestState {
    pub echo: String,
    pub foo: i32,
    pub bar: bool
}
