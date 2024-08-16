use std::path::PathBuf;

use tauri::WebviewWindow;

use util::ScreenGeometry;
use util::APP_HANDLE;
use util::USER_SETTINGS;
use windows::Win32::Foundation::BOOL;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Shell::SHAppBarMessage;
use windows::Win32::UI::WindowsAndMessaging::MoveWindow;

use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Shell::APPBARDATA;

use windows::Win32::UI::Shell::ABE_TOP;
use windows::Win32::UI::Shell::ABM_NEW;
use windows::Win32::UI::Shell::ABM_QUERYPOS;
use windows::Win32::UI::Shell::ABM_REMOVE;
use windows::Win32::UI::Shell::ABM_SETAUTOHIDEBAR;
use windows::Win32::UI::Shell::ABM_SETPOS;
use windows::Win32::UI::WindowsAndMessaging::SetMenu;
use windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrA;
use windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE;
use windows::Win32::UI::WindowsAndMessaging::USER_DEFAULT_SCREEN_DPI;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW;

static mut WINDOW_HWND: HWND = unsafe { std::mem::transmute(std::ptr::null_mut::<HWND>()) };

fn setup_window() -> Result<tauri::WebviewWindow, ()> {
  let window = tauri::WebviewWindowBuilder::new(
    unsafe { APP_HANDLE.as_ref().unwrap() },
    "menubar",
    tauri::WebviewUrl::App(PathBuf::from("/#/menubar")),
  )
  .title("Menubar")
  .transparent(true)
  .always_on_top(true)
  .decorations(false)
  .shadow(false)
  .resizable(false)
  .maximizable(false)
  .minimizable(false)
  .focused(false)
  .closable(false)
  .skip_taskbar(true)
  .build()
  .expect("Failed to build menubar window");

  Ok(window)
}

pub fn init() {
  let window = setup_window().unwrap();
  let hwnd: HWND = unsafe { std::mem::transmute(window.hwnd().unwrap().0) };
  unsafe { WINDOW_HWND = hwnd };

  unsafe {
    add().expect("Failed to add app bar");

    SetWindowLongPtrA(hwnd, GWL_EXSTYLE, WS_EX_TOOLWINDOW.0 as isize);

    // Set window size using MoveWindow
    let width = ScreenGeometry::new().width;
    let height = USER_SETTINGS.height;
    MoveWindow(hwnd, 0, 0, width, height, true).expect("Failed to set window size");
  }
}

pub fn create_round_window() -> Result<WebviewWindow, String> {
  // Create a round window
  // ...
  let window_height = USER_SETTINGS.height;
  let (width, _height) = unsafe { (ScreenGeometry::new().width, ScreenGeometry::new().height) };

  let webview_window = tauri::WebviewWindowBuilder::new(
    unsafe { APP_HANDLE.as_ref().unwrap() },
    "round-border",
    tauri::WebviewUrl::App(PathBuf::from("/#/rounded")),
  )
  .title("Round Border")
  .decorations(false)
  .resizable(false)
  .transparent(true)
  .inner_size(width.into(), 20.0)
  .position(0.0, window_height as f64)
  .shadow(false)
  .always_on_top(true)
  .skip_taskbar(true)
  .build()
  .expect("Failed to create window");

  webview_window
    .clone()
    .set_ignore_cursor_events(true)
    .unwrap();

  let hwnd: HWND = unsafe { std::mem::transmute(webview_window.hwnd().unwrap().0) };

  unsafe {
    SetMenu(hwnd, None).unwrap();
    MoveWindow(hwnd, 0, window_height as i32, width, 20, true).expect("Failed to move window");
  }

  Ok(webview_window)
}

unsafe fn get_pos(appbar_data: *mut APPBARDATA, height: i32) -> APPBARDATA {
  let geometry = ScreenGeometry::new();

  let dpi = GetDpiForWindow::<HWND>(WINDOW_HWND);
  let device_pixel_ratio = dpi as f64 / USER_DEFAULT_SCREEN_DPI as f64;
  let bar_height = height * (device_pixel_ratio as i32);

  (*appbar_data).rc = RECT {
    left: geometry.x,
    top: geometry.y,
    right: geometry.x + bar_height,
    bottom: geometry.y + bar_height,
  };

  *appbar_data
}

pub unsafe fn add() -> Result<(), &'static str> {
  let (width, height) = (ScreenGeometry::new().width, USER_SETTINGS.height);

  let default_appbar_data: *mut APPBARDATA = &mut APPBARDATA {
    cbSize: std::mem::size_of::<APPBARDATA>() as u32,
    hWnd: WINDOW_HWND,
    uEdge: ABE_TOP,
    rc: RECT::default(),
    lParam: LPARAM(0),
    uCallbackMessage: 0,
  };

  SHAppBarMessage(ABM_NEW, default_appbar_data);

  let auto_hide = false;
  let mut taskbar_pos = get_pos(default_appbar_data, height);
  if auto_hide {
    taskbar_pos.lParam = windows::Win32::Foundation::LPARAM(1);

    SHAppBarMessage(ABM_SETAUTOHIDEBAR, &mut taskbar_pos as *mut APPBARDATA);
    taskbar_pos.rc.bottom = taskbar_pos.rc.top + height;

    SHAppBarMessage(ABM_QUERYPOS, &mut taskbar_pos as *mut APPBARDATA);
    SHAppBarMessage(ABM_SETPOS, &mut taskbar_pos as *mut APPBARDATA);
    MoveWindow(WINDOW_HWND, 0, 0, width, height, BOOL::from(true)).expect("Failed to move window");
  } else {
    SHAppBarMessage(ABM_SETPOS, &mut taskbar_pos as *mut APPBARDATA);
    MoveWindow(WINDOW_HWND, 0, 0, width, height, BOOL::from(true)).expect("Failed to move window");
  }

  create_round_window().unwrap();

  Ok(())
}

pub unsafe fn remove() {
  let mut default_appbar_data = APPBARDATA {
    cbSize: std::mem::size_of::<APPBARDATA>() as u32,
    hWnd: WINDOW_HWND,
    uCallbackMessage: 0,
    uEdge: ABE_TOP,
    rc: RECT::default(),
    lParam: LPARAM(0),
  };

  SHAppBarMessage(ABM_REMOVE, &mut default_appbar_data as *mut APPBARDATA);
}
