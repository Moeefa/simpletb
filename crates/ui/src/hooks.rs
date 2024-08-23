use std::{
  sync::{LazyLock, Mutex},
  thread,
};

use icons::get_icon;
use tauri::{Emitter, Listener, PhysicalPosition, PhysicalSize};
use util::{exe_path, get_class, hide_taskbar, is_real_window, ScreenGeometry, USER_SETTINGS};
use windows::Win32::{
  Foundation::{BOOL, HWND, LPARAM},
  UI::{
    Accessibility::{SetWinEventHook, HWINEVENTHOOK},
    WindowsAndMessaging::{
      EnumWindows, GetParent, EVENT_MAX, EVENT_MIN, EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY,
      EVENT_OBJECT_HIDE, EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW, WINEVENT_OUTOFCONTEXT,
      WINEVENT_SKIPOWNPROCESS,
    },
  },
};

#[derive(Clone, serde::Serialize)]
struct Window {
  hwnd: isize,
  path: String,
  buffer: Vec<u8>,
}

static GLOBAL_APPS: LazyLock<Mutex<Vec<Window>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn init() {
  dock_hooks();
}

fn dock_hooks() {
  use crate::dock::WINDOW;
  let window = WINDOW.lock().unwrap().clone();

  active_window();
  enum_opened_windows(window.unwrap());

  fn active_window() {
    unsafe {
      SetWinEventHook(
        EVENT_MIN,
        EVENT_MAX,
        None,
        Some(win_event_hook_callback),
        0,
        0,
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
      )
    };
  }

  fn enum_opened_windows(window: tauri::WebviewWindow) {
    unsafe {
      let screen_rect = ScreenGeometry::new();
      let cloned_window = window.clone();
      cloned_window.listen("ready", move |_: tauri::Event| {
        window.show().unwrap_or_else(|_| ());
        EnumWindows(Some(enum_windows_proc), LPARAM(0)).unwrap_or_else(|_| ());

        let length = GLOBAL_APPS.lock().unwrap().len() as i32;

        let cloned_window = window.clone();
        thread::spawn(move || {
          cloned_window
            .set_position(PhysicalPosition {
              x: (screen_rect.width / 2) - ((length * 44 / 2) + 8),
              y: screen_rect.height - 51 - USER_SETTINGS.margin_bottom,
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
          .unwrap_or_else(|_| ());
      });
    }
  }

  unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
    let mut global_apps = GLOBAL_APPS.lock().unwrap();

    if is_real_window(hwnd, false) {
      let exe_path = exe_path(hwnd).unwrap_or_default();

      if exe_path.ends_with("simpletb.exe")
        || exe_path.ends_with("explorer.exe")
        || global_apps.iter().any(|window| window.hwnd == hwnd.0)
      {
        return true.into();
      }

      global_apps.push(Window {
        hwnd: hwnd.0,
        path: exe_path.clone(),
        buffer: get_icon(&exe_path).unwrap_or_else(|_| Vec::new()),
      });
    }

    true.into()
  }

  unsafe extern "system" fn win_event_hook_callback(
    _hook_handle: HWINEVENTHOOK,
    _event_id: u32,
    _window_handle: HWND,
    _object_id: i32,
    _child_id: i32,
    _thread_id: u32,
    _timestamp: u32,
  ) {
    fn update_window() {
      let binding = WINDOW.lock().unwrap();
      let window = binding.as_ref().unwrap();
      let global_apps = GLOBAL_APPS.lock().unwrap();

      let length = global_apps.len() as i32;
      let screen_rect = ScreenGeometry::new();
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
    }

    fn set_apps() {
      let global_apps = GLOBAL_APPS.lock().unwrap();
      let binding = WINDOW.lock().unwrap();
      let window = binding.as_ref().unwrap();

      window.emit("set-apps", global_apps.to_vec()).unwrap();
    }

    let mut global_apps = GLOBAL_APPS.lock().unwrap();

    match _event_id {
      EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE => {
        if "Shell_TrayWnd" == get_class(_window_handle).expect("Failed to get class") {
          hide_taskbar(true);
        }

        if global_apps
          .iter()
          .any(|window| window.hwnd == _window_handle.0)
        {
          return;
        }

        if is_real_window(_window_handle, false) {
          let exe_path = exe_path(_window_handle).unwrap_or_default();
          global_apps.push(Window {
            hwnd: _window_handle.0,
            path: exe_path.clone(),
            buffer: get_icon(&exe_path).expect("Failed to get icon"),
          });

          update_window();
          set_apps();
        }
      }
      EVENT_OBJECT_DESTROY => {
        if global_apps
          .iter()
          .any(|window| window.hwnd == _window_handle.0)
        {
          global_apps.retain(|window| window.hwnd != _window_handle.0);

          update_window();
          set_apps();
        }
      }
      EVENT_OBJECT_NAMECHANGE => {
        if global_apps
          .iter()
          .any(|window| window.hwnd == _window_handle.0)
        {
          // Todo
        } else if is_real_window(_window_handle, false) {
          let exe_path = exe_path(_window_handle).unwrap_or_default();
          global_apps.push(Window {
            hwnd: _window_handle.0,
            path: exe_path.clone(),
            buffer: get_icon(&exe_path).expect("Failed to get icon"),
          });

          update_window();
          set_apps();
        }
      }
      EVENT_OBJECT_HIDE => {
        if global_apps
          .iter()
          .any(|window| window.hwnd == _window_handle.0)
        {
          let parent = GetParent(_window_handle);
          if parent.0 != 0 {
            for app in global_apps.iter_mut() {
              if app.hwnd == _window_handle.0 {
                app.hwnd = parent.0;
                break;
              }
            }

            set_apps();
          } else {
            if !is_real_window(_window_handle, false) {
              global_apps.retain(|window| window.hwnd != _window_handle.0);

              update_window();
              set_apps();
            }
          }
        }
      }
      _ => {}
    }
  }
}
