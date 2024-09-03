use tauri::Emitter;
use tauri::Listener;

use tauri::Manager;
use tauri::PhysicalPosition;
use tauri::PhysicalSize;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::UI::Accessibility::SetWinEventHook;
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows::Win32::UI::WindowsAndMessaging::SetWindowLongA;
use windows::Win32::UI::WindowsAndMessaging::EVENT_MAX;
use windows::Win32::UI::WindowsAndMessaging::EVENT_MIN;
use windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_OUTOFCONTEXT;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_SKIPOWNPROCESS;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_NOACTIVATE;

use backdrop::enable_blur;
use util::*;

use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::thread;

use crate::hooks;

#[derive(Clone, serde::Serialize, Debug)]
pub struct Window {
  pub hwnd: isize,
  pub path: String,
  pub buffer: Vec<u8>,
}

pub static WINDOW: LazyLock<Mutex<Option<tauri::WebviewWindow>>> =
  LazyLock::new(|| Mutex::new(None));
pub static GLOBAL_APPS: LazyLock<Mutex<Vec<Window>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn init() {
  let window = setup_window().expect("Failed to setup dock window");
  let hwnd = HWND(window.hwnd().unwrap().0);
  *WINDOW.lock().unwrap() = Some(window.clone());

  // Hooks
  unsafe { setup_hooks() };

  // Styles
  hide_taskbar(true);

  // Listeners
  window.listen("mouse-out", move |_| hide());
  window.listen("mouse-in", move |_| show());
  window.once("ready", move |_| {
    thread::spawn(move || {
      enable_blur(hwnd, "#10101000", true);
      update();
      show();
    });
  });

  // No activate
  unsafe { SetWindowLongA(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32) };
}

pub fn update() {
  let binding = WINDOW.lock().unwrap();
  let window = binding.as_ref().unwrap();

  window.set_position(position()).unwrap();
  window.set_size(size()).unwrap();

  window
    .emit("set-apps", GLOBAL_APPS.lock().unwrap().to_vec())
    .expect("Failed to set apps");
}

pub fn hide() {
  let binding = WINDOW.lock().unwrap();
  let window = binding.as_ref().unwrap();
  let hwnd = HWND(window.hwnd().unwrap().0);

  window.hide().unwrap();
  unsafe { SetWindowLongA(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32) };
}

pub fn show() {
  let binding = WINDOW.lock().unwrap();
  let window = binding.as_ref().unwrap();
  let hwnd = HWND(window.hwnd().unwrap().0);

  window.show().unwrap();
  unsafe { SetWindowLongA(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32) };
}

fn setup_window() -> Result<tauri::WebviewWindow, ()> {
  let window = tauri::WebviewWindowBuilder::new(
    APP_HANDLE
      .lock()
      .unwrap()
      .as_ref()
      .unwrap_or_else(|| panic!("Failed to get app handle")),
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

// Hooks
pub unsafe fn setup_hooks() {
  active_window(); // TODO: fix this, causes a crash in window
  enum_opened_windows();
}

pub unsafe fn active_window() {
  SetWinEventHook(
    EVENT_MIN,
    EVENT_MAX,
    None,
    Some(hooks::win_event_hook_callback),
    0,
    0,
    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
  );
}

pub unsafe fn enum_opened_windows() {
  EnumWindows(Some(hooks::enum_windows_proc), LPARAM(0)).expect("Failed to enum windows")
}

// Get size and position
pub fn position() -> PhysicalPosition<i32> {
  let length = GLOBAL_APPS.lock().unwrap().len() as i32;
  let screen_rect = ScreenGeometry::new();

  PhysicalPosition {
    x: (screen_rect.width / 2) - ((length * 44 / 2) + 8),
    y: screen_rect.height - 51 - USER_SETTINGS.margin_bottom,
  }
}

pub fn size() -> PhysicalSize<i32> {
  let length = GLOBAL_APPS.lock().unwrap().len() as i32;

  PhysicalSize {
    width: (length * 44) + 8,
    height: 51,
  }
}
