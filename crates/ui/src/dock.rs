use tauri::Emitter;
use tauri::Listener;

use tauri::PhysicalPosition;
use tauri::PhysicalSize;

use ::windows::Win32::Foundation::BOOL;
use ::windows::Win32::Foundation::HINSTANCE;
use ::windows::Win32::Foundation::HWND;
use ::windows::Win32::Foundation::LPARAM;
use ::windows::Win32::UI::Accessibility::SetWinEventHook;
use ::windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use ::windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use ::windows::Win32::UI::WindowsAndMessaging::GetParent;
use ::windows::Win32::UI::WindowsAndMessaging::SetWindowLongA;
use ::windows::Win32::UI::WindowsAndMessaging::EVENT_MAX;
use ::windows::Win32::UI::WindowsAndMessaging::EVENT_MIN;
use ::windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_CREATE;
use ::windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DESTROY;
use ::windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_HIDE;
use ::windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_NAMECHANGE;
use ::windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_SHOW;
use ::windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE;
use ::windows::Win32::UI::WindowsAndMessaging::WINEVENT_OUTOFCONTEXT;
use ::windows::Win32::UI::WindowsAndMessaging::WINEVENT_SKIPOWNPROCESS;
use ::windows::Win32::UI::WindowsAndMessaging::WS_EX_NOACTIVATE;

use util::ScreenGeometry;

use backdrop::enable_blur;
use icons::get_icon;
use util::*;

use lazy_static::lazy_static;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

#[derive(Clone, serde::Serialize)]
struct Window {
  hwnd: isize,
  path: String,
  buffer: Vec<u8>,
}

lazy_static! {
  static ref WINDOW: Mutex<Option<tauri::WebviewWindow>> = Mutex::new(None);
  static ref GLOBAL_APPS: Mutex<Vec<Window>> = Mutex::new(Vec::new());
  static ref ACTIVE_WINDOW: Mutex<HWND> = Mutex::new(HWND::default());
}

#[allow(dead_code)]
type SetWinEventHookFn =
  unsafe extern "system" fn(u32, u32, *mut HINSTANCE, i32, u32, u32, u32) -> ();

pub fn init() {
  let window = setup_window().expect("Failed to setup dock window");
  *WINDOW.lock().unwrap() = Some(window.clone());

  window.listen("mouse-out", move |_msg| {
    unsafe { hide() };
  });

  window.listen("mouse-in", move |_msg| {
    unsafe { show() };
  });

  unsafe {
    let hwnd: HWND = std::mem::transmute(window.hwnd().unwrap().0);

    enum_opened_windows(window.clone());

    enable_blur(hwnd, "#10101000", true).expect("Failed to enable blur");

    active_window_event();
    hide_taskbar(true);

    SetWindowLongA(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32);

    window.show().unwrap();
  }
}

fn setup_window() -> Result<tauri::WebviewWindow, ()> {
  let window = tauri::WebviewWindowBuilder::new(
    unsafe { APP_HANDLE.as_ref().unwrap() },
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

pub unsafe fn hide() {
  let dock_hwnd: HWND =
    std::mem::transmute(WINDOW.lock().unwrap().as_ref().unwrap().hwnd().unwrap().0);

  WINDOW
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .hide()
    .expect("Failed to hide dock window");
  SetWindowLongA(dock_hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32);
}

pub unsafe fn show() {
  let dock_hwnd: HWND =
    std::mem::transmute(WINDOW.lock().unwrap().as_ref().unwrap().hwnd().unwrap().0);

  WINDOW
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .show()
    .expect("Failed to hide dock window");
  SetWindowLongA(dock_hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32);
}

pub unsafe fn active_window_event() {
  unsafe extern "system" fn win_event_hook_callback(
    _hook_handle: HWINEVENTHOOK,
    _event_id: u32,
    _window_handle: HWND,
    _object_id: i32,
    _child_id: i32,
    _thread_id: u32,
    _timestamp: u32,
  ) {
    let binding = WINDOW.lock().unwrap();
    let window = binding.as_ref().unwrap();
    let _window_hwnd = std::mem::transmute::<isize, HWND>(window.hwnd().unwrap().0);

    match _event_id {
      EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE => {
        if "Shell_TrayWnd" == get_class(_window_handle).expect("Failed to get class") {
          hide_taskbar(true);
        }

        if GLOBAL_APPS
          .lock()
          .unwrap()
          .iter()
          .any(|window| window.hwnd == _window_handle.0)
        {
          return;
        }

        if is_real_window(_window_handle, false) {
          let exe_path = exe_path(_window_handle).unwrap_or_default();
          GLOBAL_APPS.lock().unwrap().push(Window {
            hwnd: _window_handle.0,
            path: exe_path.clone(),
            buffer: get_icon(&exe_path).expect("Failed to get icon"),
          });

          let length = GLOBAL_APPS.lock().unwrap().len() as i32;
          let screen_rect = ScreenGeometry::new();
          window
            .set_position(PhysicalPosition {
              x: (screen_rect.width / 2) - ((length * 44 / 2) + 8),
              y: screen_rect.height - 51 - MARGIN_BOTTOM,
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
            .expect("Failed to emit add-app event");
        }
      }
      EVENT_OBJECT_DESTROY => {
        if GLOBAL_APPS
          .lock()
          .unwrap()
          .iter()
          .any(|window| window.hwnd == _window_handle.0)
        {
          GLOBAL_APPS
            .lock()
            .unwrap()
            .retain(|window| window.hwnd != _window_handle.0);

          let length = GLOBAL_APPS.lock().unwrap().len() as i32;
          let screen_rect = ScreenGeometry::new();

          window
            .set_position(PhysicalPosition {
              x: (screen_rect.width / 2) - ((length * 44 / 2) + 8),
              y: screen_rect.height - 51 - MARGIN_BOTTOM,
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
            .expect("Failed to emit add-app event");
        }
      }
      EVENT_OBJECT_NAMECHANGE => {
        if GLOBAL_APPS
          .lock()
          .unwrap()
          .iter()
          .any(|window| window.hwnd == _window_handle.0)
        {
          // Todo
        } else if is_real_window(_window_handle, false) {
          let exe_path = exe_path(_window_handle).unwrap_or_default();
          GLOBAL_APPS.lock().unwrap().push(Window {
            hwnd: _window_handle.0,
            path: exe_path.clone(),
            buffer: get_icon(&exe_path).expect("Failed to get icon"),
          });

          let length = GLOBAL_APPS.lock().unwrap().len() as i32;
          let screen_rect = ScreenGeometry::new();
          window
            .set_position(PhysicalPosition {
              x: (screen_rect.width / 2) - ((length * 44 / 2) + 8),
              y: screen_rect.height - 51 - MARGIN_BOTTOM,
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
            .expect("Failed to emit add-app event");
        }
      }
      EVENT_OBJECT_HIDE => {
        if GLOBAL_APPS
          .lock()
          .unwrap()
          .iter()
          .any(|window| window.hwnd == _window_handle.0)
        {
          let parent = GetParent(_window_handle);
          if parent.0 != 0 {
            for app in GLOBAL_APPS.lock().unwrap().iter_mut() {
              if app.hwnd == _window_handle.0 {
                app.hwnd = parent.0;
                break;
              }
            }

            window
              .emit("set-apps", GLOBAL_APPS.lock().unwrap().to_vec())
              .expect("Failed to emit add-app event");
          } else {
            if !is_real_window(_window_handle, false) {
              GLOBAL_APPS
                .lock()
                .unwrap()
                .retain(|window| window.hwnd != _window_handle.0);

              let length = GLOBAL_APPS.lock().unwrap().len() as i32;
              let screen_rect = ScreenGeometry::new();

              window
                .set_position(PhysicalPosition {
                  x: (screen_rect.width / 2) - ((length * 44 / 2) + 8),
                  y: screen_rect.height - 51 - MARGIN_BOTTOM,
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
                .expect("Failed to emit add-app event");
            }
          }
        }
      }
      _ => {}
    }
  }

  SetWinEventHook(
    EVENT_MIN,
    EVENT_MAX,
    None,
    Some(win_event_hook_callback),
    0,
    0,
    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
  );
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
  if is_real_window(hwnd, false) {
    let exe_path = exe_path(hwnd).unwrap_or_default();

    if exe_path.ends_with("simpletb.exe")
      || exe_path.ends_with("explorer.exe")
      || GLOBAL_APPS
        .lock()
        .unwrap()
        .iter()
        .any(|window| window.hwnd == hwnd.0)
    {
      return true.into();
    }

    GLOBAL_APPS.lock().unwrap().push(Window {
      hwnd: hwnd.0,
      path: exe_path.clone(),
      buffer: get_icon(&exe_path).expect("Failed to get icon"),
    });
  }

  true.into()
}

pub unsafe fn enum_opened_windows(window: tauri::WebviewWindow) {
  let screen_rect = ScreenGeometry::new();
  let cloned_window = window.clone();
  cloned_window.listen("ready", move |_: tauri::Event| {
    window.show().expect("Failed to show window");
    EnumWindows(Some(enum_windows_proc), LPARAM(0)).expect("Failed to enum windows");

    let length = GLOBAL_APPS.lock().unwrap().len() as i32;

    let cloned_window = window.clone();
    thread::spawn(move || {
      cloned_window
        .set_position(PhysicalPosition {
          x: (screen_rect.width / 2) - ((length * 44 / 2) + 8),
          y: screen_rect.height - 51 - MARGIN_BOTTOM,
        })
        .unwrap();

      cloned_window
        .set_size(PhysicalSize {
          width: (length * 44) + 8,
          height: 51,
        })
        .unwrap();
    });

    let cloned_window = window.clone();
    cloned_window
      .emit("set-apps", GLOBAL_APPS.lock().unwrap().to_vec())
      .expect("Failed to emit add-app event");
  });
}
