use std::sync::{LazyLock, Mutex};

use tauri::AppHandle;

pub static APP_HANDLE: LazyLock<Mutex<Option<AppHandle>>> = LazyLock::new(|| Mutex::new(None));
