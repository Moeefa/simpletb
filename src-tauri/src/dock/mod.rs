use std::{ops::BitAnd, ptr, thread};

use tauri::Manager;
use windows::Win32::{
  Foundation::{BOOL, HINSTANCE, HWND, LPARAM},
  UI::{
    Accessibility::{SetWinEventHook, HWINEVENTHOOK},
    WindowsAndMessaging::{
      EnumWindows, GetParent, GetWindow, GetWindowInfo, GetWindowLongA, IsIconic, IsWindowVisible,
      MoveWindow, SetWindowLongA, EVENT_MAX, EVENT_MIN, EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY,
      EVENT_OBJECT_FOCUS, EVENT_OBJECT_HIDE, EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW,
      EVENT_SYSTEM_FOREGROUND, GET_WINDOW_CMD, GWL_EXSTYLE, GWL_STYLE, GW_OWNER, WINDOW_EX_STYLE,
      WINDOW_STYLE, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WS_EX_APPWINDOW,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_VISIBLE,
    },
  },
};

use crate::utils::{blur_window::enable_blur, icons_to_buff, Utils};
use crate::ScreenGeometry;
use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Clone)]
pub struct Dock {
  handle: tauri::AppHandle,
}

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

const MARGIN_BOTTOM: i32 = 5;

impl Dock {
  pub fn new(handle: tauri::AppHandle) -> Self {
    Self { handle }
  }

  fn setup_window(&self) -> Result<tauri::WebviewWindow, ()> {
    let window = tauri::WebviewWindowBuilder::new(
      &self.handle,
      "dock",
      tauri::WebviewUrl::App("/src/dock/index.html".into()),
    )
    .title("dock")
    .transparent(true)
    .always_on_top(true)
    .decorations(false)
    .shadow(true)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .skip_taskbar(true)
    .build()
    .expect("Failed to build dock window");

    Ok(window)
  }

  pub fn init(&self) -> Result<(), &'static str> {
    let window = self.setup_window().expect("Failed to setup dock window");
    *WINDOW.lock().unwrap() = Some(window.clone());
    window.hide().expect("Failed to hide dock window");

    unsafe {
      let hwnd: HWND = std::mem::transmute(window.hwnd().unwrap().0);

      enable_blur(hwnd, "#10101000", true).expect("Failed to enable blur");

      self.enum_opened_windows(window.clone());

      window.show().expect("Failed to show dock window");

      self.active_window_event();
      Utils::hide_taskbar(true);

      SetWindowLongA(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32);

      Ok(())
    }
  }

  pub unsafe fn hide(&self) {
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

  pub unsafe fn show(&self) {
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

  pub unsafe fn active_window_event(&self) {
    unsafe extern "system" fn win_event_hook_callback(
      _hook_handle: HWINEVENTHOOK,
      _event_id: u32,
      _window_handle: HWND,
      _object_id: i32,
      _child_id: i32,
      _thread_id: u32,
      _timestamp: u32,
    ) {
      let window = WINDOW.lock().unwrap();
      let window_hwnd =
        std::mem::transmute::<isize, HWND>(window.as_ref().unwrap().hwnd().unwrap().0);

      match _event_id {
        EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE => {
          if "Shell_TrayWnd" == Utils::get_class(_window_handle).expect("Failed to get class") {
            Utils::hide_taskbar(true);
          }

          if GLOBAL_APPS
            .lock()
            .unwrap()
            .iter()
            .any(|window| window.hwnd == _window_handle.0)
          {
            return;
          }

          if Utils::is_real_window(_window_handle, false) {
            let exe_path = Utils::exe_path(_window_handle).unwrap_or_default();
            GLOBAL_APPS.lock().unwrap().push(Window {
              hwnd: _window_handle.0,
              path: exe_path.clone(),
              buffer: icons_to_buff::get_icon(&exe_path, 32).expect("Failed to get icon"),
            });

            let length = GLOBAL_APPS.lock().unwrap().len() as i32;
            let screen_rect = ScreenGeometry::new();
            thread::spawn(move || {
              MoveWindow(
                window_hwnd,
                (screen_rect.width / 2) - ((length * 44 / 2) + 8),
                screen_rect.height - 51 - MARGIN_BOTTOM,
                (length * 44) + 8,
                51,
                true,
              )
              .expect("Failed to move dock window");
            });

            window
              .as_ref()
              .unwrap()
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

            thread::spawn(move || {
              MoveWindow(
                window_hwnd,
                (screen_rect.width / 2) - ((length * 44 / 2) + 8),
                screen_rect.height - 51 - MARGIN_BOTTOM,
                (length * 44) + 8,
                51,
                true,
              )
              .expect("Failed to move dock window");
            });

            window
              .as_ref()
              .unwrap()
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
          } else if Utils::is_real_window(_window_handle, false) {
            let exe_path = Utils::exe_path(_window_handle).unwrap_or_default();
            GLOBAL_APPS.lock().unwrap().push(Window {
              hwnd: _window_handle.0,
              path: exe_path.clone(),
              buffer: icons_to_buff::get_icon(&exe_path, 32).expect("Failed to get icon"),
            });

            let length = GLOBAL_APPS.lock().unwrap().len() as i32;
            let screen_rect = ScreenGeometry::new();
            thread::spawn(move || {
              MoveWindow(
                window_hwnd,
                (screen_rect.width / 2) - ((length * 44 / 2) + 8),
                screen_rect.height - 51 - MARGIN_BOTTOM,
                (length * 44) + 8,
                51,
                true,
              )
              .expect("Failed to move dock window");
            });

            window
              .as_ref()
              .unwrap()
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
                .as_ref()
                .unwrap()
                .emit("set-apps", GLOBAL_APPS.lock().unwrap().to_vec())
                .expect("Failed to emit add-app event");
            } else {
              if !Utils::is_real_window(_window_handle, false) {
                GLOBAL_APPS
                  .lock()
                  .unwrap()
                  .retain(|window| window.hwnd != _window_handle.0);

                let length = GLOBAL_APPS.lock().unwrap().len() as i32;
                let screen_rect = ScreenGeometry::new();

                thread::spawn(move || {
                  MoveWindow(
                    window_hwnd,
                    (screen_rect.width / 2) - ((length * 44 / 2) + 8),
                    screen_rect.height - 51 - MARGIN_BOTTOM,
                    (length * 44) + 8,
                    51,
                    true,
                  )
                  .expect("Failed to move dock window");
                });

                window
                  .as_ref()
                  .unwrap()
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
    if Utils::is_real_window(hwnd, false) {
      let exe_path = Utils::exe_path(hwnd).unwrap_or_default();

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
        buffer: icons_to_buff::get_icon(&exe_path, 32).expect("Failed to get icon"),
      });
    }

    true.into()
  }

  pub unsafe fn enum_opened_windows(&self, window: tauri::WebviewWindow) {
    let cloned_window = window.clone();
    let screen_rect = ScreenGeometry::new();
    let dock_hwnd: HWND = std::mem::transmute(window.hwnd().unwrap().0);
    window.listen("ready", move |_: tauri::Event| {
      EnumWindows(Some(Self::enum_windows_proc), LPARAM(0)).expect("Failed to enum windows");

      let length = GLOBAL_APPS.lock().unwrap().len() as i32;
      thread::spawn(move || {
        MoveWindow(
          dock_hwnd,
          (screen_rect.width / 2) - ((length * 44 / 2) + 8),
          screen_rect.height - 51 - MARGIN_BOTTOM,
          (length * 44) + 8,
          51,
          true,
        )
        .expect("Failed to move dock window");
      });

      cloned_window
        .emit("set-apps", GLOBAL_APPS.lock().unwrap().to_vec())
        .expect("Failed to emit add-app event");
    });
  }
}
