use tauri::Listener;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SetWindowLongA;
use windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_NOACTIVATE;

use backdrop::enable_blur;
use util::*;

use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

pub static WINDOW: LazyLock<Mutex<Option<tauri::WebviewWindow>>> =
  LazyLock::new(|| Mutex::new(None));

pub fn init() {
  let window = setup_window().expect("Failed to setup dock window");
  let hwnd: HWND = HWND(window.hwnd().unwrap().0);
  *WINDOW.lock().unwrap() = Some(window.clone());

  window.listen("mouse-out", move |_msg| hide());
  window.listen("mouse-in", move |_msg| show());

  enable_blur(hwnd, "#10101000", true);

  hide_taskbar(true);

  unsafe { SetWindowLongA(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32) };

  window.show().unwrap();
}

pub fn hide() {
  let binding = WINDOW.lock().unwrap();
  let window = binding.as_ref().unwrap();
  let dock_hwnd: HWND = HWND(window.hwnd().unwrap().0);

  window.hide().expect("Failed to hide dock window");
  unsafe { SetWindowLongA(dock_hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32) };
}

pub fn show() {
  let binding = WINDOW.lock().unwrap();
  let window = binding.as_ref().unwrap();
  let dock_hwnd: HWND = HWND(window.hwnd().unwrap().0);

  window.show().expect("Failed to hide dock window");
  unsafe { SetWindowLongA(dock_hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32) };
}

fn setup_window() -> Result<tauri::WebviewWindow, ()> {
  let window = tauri::WebviewWindowBuilder::new(
    APP_HANDLE.lock().unwrap().as_ref().unwrap_or_else(|| {
      panic!("Failed to get app handle");
    }),
    "dock",
    tauri::WebviewUrl::App(PathBuf::from("/#/dock")),
  )
  .title("Dock")
  .transparent(true)
  .always_on_top(true)
  .decorations(false)
  .shadow(true)
  .resizable(false)
  .maximizable(false)
  .minimizable(false)
  .skip_taskbar(true)
  .visible(false)
  .build()
  .expect("Failed to build dock window");

  Ok(window)
}
