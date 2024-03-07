tauri_interop::host_usage! {
    use tauri_interop::command::TauriAppHandle;
}

#[tauri_interop::command]
pub fn stop_application(handle: TauriAppHandle) {
    handle.exit(0)
}

tauri_interop::collect_commands!();
