use tauri::Manager;
use tauri::WebviewWindow;

use windows::Win32::Foundation::BOOL;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Shell::SHAppBarMessage;
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
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
use windows::Win32::UI::WindowsAndMessaging::USER_DEFAULT_SCREEN_DPI;

pub struct ScreenGeometry {
  pub x: i32,
  pub y: i32,
  pub height: i32,
  pub width: i32,
}

impl ScreenGeometry {
  const fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      height: 0,
      width: 0,
    }
  }

  pub unsafe fn new() -> Self {
    let mut screen_rect = RECT::default();
    let mut geometry = Self::default();

    GetWindowRect(GetDesktopWindow(), &mut screen_rect).unwrap();
    geometry.x = screen_rect.left;
    geometry.y = screen_rect.top;
    geometry.width = screen_rect.right;
    geometry.height = screen_rect.bottom;

    geometry
  }
}

pub struct AppBar {
  hwnd: HWND,
  window: WebviewWindow,
}

impl Clone for AppBar {
  fn clone(&self) -> Self {
    // Implement logic to create a new instance of AppBar with the same data
    // For example:
    AppBar::new(&self.window)
  }
}

impl AppBar {
  pub fn new(window: &WebviewWindow) -> Self {
    Self {
      hwnd: unsafe { std::mem::transmute(window.hwnd().unwrap().0) },
      window: window.clone(),
    }
  }

  unsafe fn get_bar_pos(&self, appbar_data: *mut APPBARDATA, height: i32) -> APPBARDATA {
    let geometry = ScreenGeometry::new();

    let dpi = GetDpiForWindow::<HWND>(self.hwnd);
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

  pub unsafe fn add_app_bar(&self) -> Result<(), &'static str> {
    let (width, height) = (
      ScreenGeometry::new().width,
      self
        .window
        .config()
        .app
        .windows
        .iter()
        .find(|e| e.label == "main")
        .expect("Couldn't find main window")
        .height as i32,
    );

    let default_appbar_data: *mut APPBARDATA = &mut APPBARDATA {
      cbSize: std::mem::size_of::<APPBARDATA>() as u32,
      hWnd: self.hwnd,
      uEdge: ABE_TOP,
      rc: RECT::default(),
      lParam: LPARAM(0),
      uCallbackMessage: 0,
    };

    SHAppBarMessage(ABM_NEW, default_appbar_data);

    let auto_hide = false;
    let mut taskbar_pos = self.get_bar_pos(default_appbar_data, height);
    if auto_hide {
      taskbar_pos.lParam = windows::Win32::Foundation::LPARAM(1);

      SHAppBarMessage(ABM_SETAUTOHIDEBAR, &mut taskbar_pos as *mut APPBARDATA);
      taskbar_pos.rc.bottom = taskbar_pos.rc.top + height;

      SHAppBarMessage(ABM_QUERYPOS, &mut taskbar_pos as *mut APPBARDATA);
      SHAppBarMessage(ABM_SETPOS, &mut taskbar_pos as *mut APPBARDATA);
      MoveWindow(self.hwnd, 0, 0, width, height, BOOL::from(true)).expect("Failed to move window");
    } else {
      SHAppBarMessage(ABM_SETPOS, &mut taskbar_pos as *mut APPBARDATA);
      MoveWindow(self.hwnd, 0, 0, width, height, BOOL::from(true)).expect("Failed to move window");
    }

    Ok(())
  }

  pub unsafe fn remove_app_bar(&self) {
    let mut default_appbar_data = APPBARDATA {
      cbSize: std::mem::size_of::<APPBARDATA>() as u32,
      hWnd: self.hwnd,
      uCallbackMessage: 0,
      uEdge: ABE_TOP,
      rc: RECT::default(),
      lParam: LPARAM(0),
    };

    SHAppBarMessage(ABM_REMOVE, &mut default_appbar_data as *mut APPBARDATA);
  }
}
