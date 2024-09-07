use std::sync::{LazyLock, Mutex, Arc};

use tauri::AppHandle;

pub static APP_HANDLE: LazyLock<Arc<Mutex<Option<AppHandle>>>> = LazyLock::new(|| Arc::new(Mutex::new(None)));
