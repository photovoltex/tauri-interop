mod cmd;
mod other_cmd;

pub use cmd::*;
pub use other_cmd::*;

tauri_interop::collect_commands!();
