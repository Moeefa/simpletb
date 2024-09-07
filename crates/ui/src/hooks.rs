use crate::dock::*;

use std::thread;

use icons::get_icon;
use util::*;

use tauri::Emitter;

use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::WindowsAndMessaging::GetParent;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_CREATE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DESTROY;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_HIDE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_NAMECHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_SHOW;

pub unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
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

pub unsafe extern "system" fn win_event_hook_callback(
  _hook_handle: HWINEVENTHOOK,
  _event_id: u32,
  _window_handle: HWND,
  _object_id: i32,
  _child_id: i32,
  _thread_id: u32,
  _timestamp: u32,
) {
  let mut global_apps = GLOBAL_APPS.lock().unwrap();

  fn update() {
    thread::spawn(move || {
      let binding = WINDOW.lock().unwrap();
      let window = binding.as_ref().unwrap();

      window.set_position(position()).unwrap();
      window.set_size(size()).unwrap();

      window
        .emit("set-apps", GLOBAL_APPS.lock().unwrap().to_vec())
        .expect("Failed to set apps");
    });
  }

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

        update();
      }
    }
    EVENT_OBJECT_DESTROY => {
      if global_apps
        .iter()
        .any(|window| window.hwnd == _window_handle.0)
      {
        global_apps.retain(|window| window.hwnd != _window_handle.0);

        update();
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

        update();
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

          update();
        } else {
          if !is_real_window(_window_handle, false) {
            global_apps.retain(|window| window.hwnd != _window_handle.0);

            update();
          }
        }
      }
    }
    _ => {}
  }
}
