use std::path::Path;

use windows::core::PCWSTR;
use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::Threading::OpenProcess;
use windows::Win32::System::Threading::QueryFullProcessImageNameW;
use windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS;
use windows::Win32::System::Threading::PROCESS_NAME_WIN32;
use windows::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
use windows::Win32::UI::Shell::SHAppBarMessage;
use windows::Win32::UI::Shell::ABM_SETSTATE;
use windows::Win32::UI::Shell::ABS_ALWAYSONTOP;
use windows::Win32::UI::Shell::ABS_AUTOHIDE;
use windows::Win32::UI::Shell::APPBARDATA;
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
use windows::Win32::UI::WindowsAndMessaging::GetCursorInfo;
use windows::Win32::UI::WindowsAndMessaging::GetParent;
use windows::Win32::UI::WindowsAndMessaging::GetWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindowLongW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::Win32::UI::WindowsAndMessaging::CURSORINFO;
use windows::Win32::UI::WindowsAndMessaging::CURSOR_SHOWING;
use windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE;
use windows::Win32::UI::WindowsAndMessaging::GWL_STYLE;
use windows::Win32::UI::WindowsAndMessaging::GW_OWNER;
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_APPWINDOW;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_NOACTIVATE;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW;
use windows::Win32::UI::WindowsAndMessaging::WS_VISIBLE;

use crate::{AppError, Result};

pub fn is_cursor_visible() -> bool {
  unsafe {
    let mut cursor_info = CURSORINFO {
      cbSize: std::mem::size_of::<CURSORINFO>() as u32,
      ..Default::default()
    };

    GetCursorInfo(&mut cursor_info as *mut CURSORINFO).unwrap();
    cursor_info.flags == CURSOR_SHOWING
  }
}

pub fn get_class(hwnd: HWND) -> Result<String> {
  let mut text = [0u16; 512];
  let len = unsafe { GetClassNameW(hwnd, &mut text) };
  let length = len as usize;
  String::from_utf16(&text[..length]).map_err(|err| AppError::Utf16(err))
}

pub fn hide_taskbar(hide: bool) {
  let lparam = if hide {
    LPARAM(ABS_AUTOHIDE as isize)
  } else {
    LPARAM(ABS_ALWAYSONTOP as isize)
  };
  let cmdshow = if hide { SW_HIDE } else { SW_SHOWNORMAL };

  let name = format!("Shell_TrayWnd\0")
    .encode_utf16()
    .collect::<Vec<_>>();
  let mut app_bar = APPBARDATA {
    cbSize: std::mem::size_of::<APPBARDATA>() as u32,
    hWnd: unsafe { FindWindowW(PCWSTR(name.as_ptr()), PCWSTR::null()) },
    ..unsafe { std::mem::zeroed() }
  };

  if app_bar.hWnd.0 != 0 {
    app_bar.lParam = lparam;
    unsafe {
      SHAppBarMessage(ABM_SETSTATE, &mut app_bar);
      ShowWindow(app_bar.hWnd, cmdshow);
    }
  }
}

pub fn get_ex_styles(hwnd: HWND) -> WINDOW_EX_STYLE {
  WINDOW_EX_STYLE(unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 })
}

pub fn get_styles(hwnd: HWND) -> WINDOW_STYLE {
  WINDOW_STYLE(unsafe { GetWindowLongW(hwnd, GWL_STYLE) as u32 })
}

fn close_handle(handle: HANDLE) -> Result<()> {
  unsafe { CloseHandle(handle).map_err(|err| AppError::Windows(err)) }
}

fn open_process(
  access_rights: PROCESS_ACCESS_RIGHTS,
  inherit_handle: bool,
  process_id: u32,
) -> Result<HANDLE> {
  unsafe {
    OpenProcess(access_rights, inherit_handle, process_id).map_err(|err| AppError::Windows(err))
  }
}

fn process_handle(process_id: u32) -> Result<HANDLE> {
  open_process(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
}

pub fn window_thread_process_id(hwnd: HWND) -> (u32, u32) {
  let mut process_id = 0;
  let thread_id = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };
  (process_id, thread_id)
}

pub fn exe_path(hwnd: HWND) -> Result<String> {
  let (process_id, _) = window_thread_process_id(hwnd);
  let process_handle = process_handle(process_id)?;
  let mut lpdw_size: u32 = MAX_PATH;
  let mut process_path_raw = vec![0; MAX_PATH as usize];
  let process_path_pwstr = PWSTR::from_raw(process_path_raw.as_mut_ptr());

  let process_path = unsafe {
    QueryFullProcessImageNameW(
      process_handle,
      PROCESS_NAME_WIN32,
      process_path_pwstr,
      &mut lpdw_size,
    )
    .unwrap();

    close_handle(process_handle).unwrap();

    process_path_pwstr.to_string().map_err(|_| ())
  };

  Ok(
    Path::new(&process_path.unwrap())
      .to_path_buf()
      .to_str()
      .unwrap_or_default()
      .into(),
  )
}

pub fn is_window_visible(hwnd: HWND) -> bool {
  unsafe { IsWindowVisible(hwnd) }.into()
}

pub fn get_window_text(hwnd: HWND) -> String {
  let mut text = [0u16; 512];
  let len = unsafe { GetWindowTextW(hwnd, &mut text) };
  let length = len as usize;
  String::from_utf16(&text[..length]).unwrap_or_default()
}

pub fn is_real_window(hwnd: HWND, ignore_frame: bool) -> bool {
  if !is_window_visible(hwnd) {
    return false;
  }

  if unsafe { GetParent(hwnd) }.0 != 0 {
    return false;
  }

  let ex_style = get_ex_styles(hwnd);
  let style = get_styles(hwnd);
  let owner = unsafe { GetWindow(hwnd, GW_OWNER) };

  if !style.contains(WS_VISIBLE)
    && (!ex_style.contains(WS_EX_APPWINDOW)
      || (owner.0 != 0 && ex_style.contains(WS_EX_TOOLWINDOW)))
  {
    return false;
  }

  if (ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_NOACTIVATE))
    && !ex_style.contains(WS_EX_APPWINDOW)
  {
    return false;
  }

  if let Ok(exe_path) = exe_path(hwnd) {
    if exe_path.starts_with("C:\\Windows\\SystemApps")
      || (!ignore_frame && exe_path.ends_with("ApplicationFrameHost.exe"))
    {
      return false;
    }
  }

  !get_window_text(hwnd).is_empty()
}
