use std::path::PathBuf;

use util::ScreenGeometry;
use util::APP_HANDLE;

use tauri::Emitter;
use tauri::Listener;
use tauri::Manager;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::MoveWindow;

pub fn init() {
  let window = setup_window().expect("Failed to setup dock window");
  let hwnd: HWND = unsafe { std::mem::transmute(window.hwnd().unwrap().0) };

  unsafe {
    let screen_rect = ScreenGeometry::new();
    // SetWindowPos(hwnd, HWND_TOPMOST, 0, screen_rect.height, screen_rect.width, 2, SWP_NOSIZE | SWP_NOMOVE).expect("Failed to set window position");
    MoveWindow(hwnd, 0, screen_rect.height - 2, screen_rect.width, 2, true)
      .expect("Failed to move window");
  }

  let cloned_window = window.clone();
  cloned_window.listen("mouse-in", move |_msg| {
    window
      .app_handle()
      .emit("hide-taskbar", ())
      .unwrap_or_else(|_| ());
  });
}

pub fn setup_window() -> Result<tauri::WebviewWindow, ()> {
  let window = tauri::WebviewWindowBuilder::new(
    unsafe { APP_HANDLE.as_ref().unwrap() },
    "hitbox",
    tauri::WebviewUrl::App(PathBuf::from("/#/hitbox")),
  )
  .title("Hitbox")
  .transparent(true)
  .always_on_top(true)
  .decorations(false)
  .resizable(false)
  .maximizable(false)
  .minimizable(false)
  .shadow(false)
  .skip_taskbar(true)
  .build()
  .expect("Failed to build hitbox window");

  Ok(window)
}
