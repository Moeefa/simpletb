use tauri::Emitter;
use tauri::Listener;

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

use crate::hooks;
use crate::hooks::GLOBAL_APPS;

pub static WINDOW: LazyLock<Mutex<Option<tauri::WebviewWindow>>> =
  LazyLock::new(|| Mutex::new(None));

pub fn init() {
  let window = setup_window().expect("Failed to setup dock window");
  let hwnd: HWND = HWND(window.hwnd().unwrap().0);
  *WINDOW.lock().unwrap() = Some(window.clone());

  setup_hooks();

  window.listen("mouse-out", move |_msg| hide());
  window.listen("mouse-in", move |_msg| show());

  enable_blur(hwnd, &String::from("#10101000"), true);

  hide_taskbar(true);

  unsafe { SetWindowLongA(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32) };
}

pub fn update() {
  let screen_rect = ScreenGeometry::new();
  let binding = WINDOW.lock().unwrap();
  let window = binding.as_ref().unwrap();
  let length = GLOBAL_APPS.lock().unwrap().len() as i32;

  window
    .set_position(PhysicalPosition {
      x: (screen_rect.width / 2) - ((length * 44 / 2) + 8),
      y: screen_rect.height - 51 - USER_SETTINGS.margin_bottom,
    })
    .unwrap();

  window
    .set_size(PhysicalSize {
      width: (length * 44) + 8,
      height: 51,
    })
    .unwrap();

  window
    .emit("set-apps", GLOBAL_APPS.lock().unwrap().to_vec())
    .unwrap_or_else(|_| ());
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

// Hooks
// ...
// TODO: fix this, causes a crash in window
pub fn setup_hooks() {
  // active_window();
  // enum_opened_windows();
}

pub fn active_window() {
  unsafe {
    SetWinEventHook(
      EVENT_MIN,
      EVENT_MAX,
      None,
      Some(hooks::win_event_hook_callback),
      0,
      0,
      WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
    )
  };
}

pub fn enum_opened_windows() {
  let binding = WINDOW.lock().unwrap();
  let window = binding.as_ref().unwrap().clone();

  window.listen("ready", move |_: tauri::Event| {
    show();

    unsafe { EnumWindows(Some(hooks::enum_windows_proc), LPARAM(0)).unwrap_or_else(|_| ()) };

    update();
  });
}
