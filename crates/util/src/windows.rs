use windows::{
  core::{PCWSTR, PWSTR},
  Win32::{
      Foundation::{CloseHandle, HANDLE, HWND, LPARAM},
      System::Threading::{
          OpenProcess, QueryFullProcessImageNameW, PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32,
          PROCESS_QUERY_INFORMATION,
      },
      UI::{
          Shell::{SHAppBarMessage, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA},
          WindowsAndMessaging::{
              FindWindowW, GetClassNameW, GetParent, GetWindow, GetWindowLongW, GetWindowTextW,
              GetWindowThreadProcessId, IsWindowVisible, ShowWindow, GWL_EXSTYLE, GWL_STYLE, GW_OWNER,
              SW_HIDE, SW_SHOWNORMAL, WINDOW_EX_STYLE, WINDOW_STYLE, WS_EX_APPWINDOW,
              WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_VISIBLE,
          },
      },
  },
};

use handler::{AppError, Result};

pub fn get_class(hwnd: HWND) -> Result<String> {
    let mut text = [0u16; 512];
    let len = unsafe { GetClassNameW(hwnd, &mut text) };
    let length = len as usize;
    String::from_utf16(&text[..length]).map_err(|err| AppError::Utf16(err))
}

pub fn hide_taskbar(hide: bool) {
    let lparam = if hide { LPARAM(ABS_AUTOHIDE as isize) } else { LPARAM(ABS_ALWAYSONTOP as isize) };
    let cmdshow = if hide { SW_HIDE } else { SW_SHOWNORMAL };

    let name = format!("Shell_TrayWnd\0").encode_utf16().collect::<Vec<_>>();
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

fn open_process(access_rights: PROCESS_ACCESS_RIGHTS, inherit_handle: bool, process_id: u32) -> Result<HANDLE> {
    unsafe { OpenProcess(access_rights, inherit_handle, process_id).map_err(|err| AppError::Windows(err)) }
}

fn process_handle(process_id: u32) -> Result<HANDLE> {
    open_process(PROCESS_QUERY_INFORMATION, false, process_id)
}

pub fn window_thread_process_id(hwnd: HWND) -> (u32, u32) {
    let mut process_id = 0;
    let thread_id = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };
    (process_id, thread_id)
}

pub fn exe_path(hwnd: HWND) -> Result<String> {
    let mut len = 512u32;
    let mut path = vec![0u16; len as usize];

    let (process_id, _) = window_thread_process_id(hwnd);
    let handle = process_handle(process_id)?;
    unsafe {
        QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(path.as_mut_ptr()), &mut len)
            .map_err(|err| AppError::Windows(err))?;
    }
    close_handle(handle)?;

    String::from_utf16(&path[..len as usize]).map_err(|err| AppError::Utf16(err))
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
