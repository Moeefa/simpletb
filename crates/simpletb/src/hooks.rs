use std::sync::LazyLock;
use std::sync::Mutex;

use active_win_pos_rs::get_active_window;
use active_win_pos_rs::ActiveWindow;

use regex::Regex;

use tauri::Emitter;

use icons::get_icon;
use util::is_cursor_visible;
use util::ScreenGeometry;
use util::APP_HANDLE;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::SetWinEventHook;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_FOCUS;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_OUTOFCONTEXT;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_SKIPOWNPROCESS;

static PREV_WINDOW: LazyLock<Mutex<ActiveWindow>> =
  LazyLock::new(|| Mutex::new(ActiveWindow::default()));
static IS_FULLSCREEN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
  buffer: Vec<u8>,
  hwnd: isize,
}

// Function to setup event listeners
pub fn init() {
  // Hook window focus event
  let _ = unsafe {
    SetWinEventHook(
      EVENT_SYSTEM_FOREGROUND,
      EVENT_SYSTEM_FOREGROUND,
      None,
      Some(win_event_hook_callback),
      0,
      0,
      WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
    )
  };

  // Hook window focus change event
  let _ = unsafe {
    SetWinEventHook(
      EVENT_OBJECT_FOCUS,
      EVENT_OBJECT_FOCUS,
      None,
      Some(win_event_hook_callback),
      0,
      0,
      WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
    )
  };
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
  let binding = APP_HANDLE.lock().unwrap();
  let app_handle = binding.as_ref().unwrap_or_else(|| {
    panic!("Failed to get app handle");
  });

  match _event_id {
    EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND => match get_active_window() {
      Ok(active_window) => {
        let active_window_hwnd = Regex::new(r"[^0-9.]")
          .unwrap()
          .replace_all(&active_window.window_id, "")
          .to_string()
          .parse::<isize>()
          .unwrap();

        let (width, height) = (active_window.position.width, active_window.position.height);
        let screen = ScreenGeometry::new();
        if width == screen.width as f64
          && height == screen.height as f64
          && !is_cursor_visible()
          && !*IS_FULLSCREEN.lock().unwrap()
        {
          println!("{} {:?}", active_window.app_name, is_cursor_visible());
          app_handle.emit("app-fullscreen", ()).unwrap_or_else(|_| ());
        } else {
          if active_window.app_name != "Windows Explorer" {
            app_handle
              .emit("app-not-fullscreen", ())
              .unwrap_or_else(|_| ());
          }
        }

        if active_window.app_name != PREV_WINDOW.lock().unwrap().app_name.as_str()
          && active_window.app_name != env!("CARGO_PKG_DESCRIPTION")
        {
          let icon = get_icon(&active_window.process_path.to_str().ok_or("").unwrap()).unwrap();

          *PREV_WINDOW.lock().unwrap() = active_window.clone();
          app_handle
            .emit(
              "active-window",
              Payload {
                message: active_window.app_name,
                buffer: icon,
                hwnd: active_window_hwnd,
              },
            )
            .unwrap_or_else(|_| ());
        }
      }
      Err(_err) => {
        *PREV_WINDOW.lock().unwrap() = ActiveWindow::default();
        app_handle
          .emit_to(
            "menubar",
            "active-window",
            Payload {
              message: "Windows Explorer".to_owned(),
              buffer: Vec::new(),
              hwnd: -1,
            },
          )
          .unwrap_or_else(|_| ());
      }
    },
    _ => {}
  }
}
