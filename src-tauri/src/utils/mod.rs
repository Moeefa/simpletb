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
        SHOW_WINDOW_CMD, SW_HIDE, SW_SHOWNORMAL, WINDOW_EX_STYLE, WINDOW_STYLE, WS_EX_APPWINDOW,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_VISIBLE,
      },
    },
  },
};

pub mod blur_window;
pub mod error_handler;
pub mod icons_to_buff;

use error_handler::Result;

pub struct Utils {}
impl Utils {
  pub fn get_class(hwnd: HWND) -> Result<String, &'static str> {
    let mut text: [u16; 512] = [0; 512];
    let len = unsafe { GetClassNameW(hwnd, &mut text) };
    let length = usize::try_from(len).unwrap_or(0);
    Ok(String::from_utf16(&text[..length]).expect("Failed to convert to string"))
  }

  pub fn hide_taskbar(hide: bool) {
    let lparam: LPARAM;
    let cmdshow: SHOW_WINDOW_CMD;
    if hide {
      lparam = LPARAM(ABS_AUTOHIDE as isize);
      cmdshow = SW_HIDE;
    } else {
      lparam = LPARAM(ABS_ALWAYSONTOP as isize);
      cmdshow = SW_SHOWNORMAL;
    }

    let name: Vec<u16> = format!("Shell_TrayWnd\0").encode_utf16().collect();
    let mut ap_bar: APPBARDATA = unsafe { std::mem::zeroed() };

    ap_bar.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
    ap_bar.hWnd = unsafe { FindWindowW(PCWSTR(name.as_ptr()), PCWSTR::null()) };

    if ap_bar.hWnd.0 != 0 {
      ap_bar.lParam = lparam;
      unsafe {
        SHAppBarMessage(ABM_SETSTATE, &mut ap_bar as *mut APPBARDATA);
        ShowWindow(ap_bar.hWnd, cmdshow);
      }
    }
  }

  pub fn get_ex_styles(hwnd: HWND) -> WINDOW_EX_STYLE {
    WINDOW_EX_STYLE(unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32)
  }

  pub fn get_styles(hwnd: HWND) -> WINDOW_STYLE {
    WINDOW_STYLE(unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32)
  }

  fn close_handle(handle: HANDLE) -> Result<()> {
    unsafe {
      CloseHandle(handle).expect("Failed to close handle");
    }
    Ok(())
  }

  fn open_process(
    access_rights: PROCESS_ACCESS_RIGHTS,
    inherit_handle: bool,
    process_id: u32,
  ) -> Result<HANDLE> {
    unsafe { Ok(OpenProcess(access_rights, inherit_handle, process_id)?) }
  }

  fn process_handle(process_id: u32) -> Result<HANDLE> {
    Self::open_process(PROCESS_QUERY_INFORMATION, false, process_id)
  }

  pub fn window_thread_process_id(hwnd: HWND) -> (u32, u32) {
    let mut process_id: u32 = 0;

    // Behaviour is undefined if an invalid HWND is given
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
    let thread_id =
      unsafe { GetWindowThreadProcessId(hwnd, Option::from(std::ptr::addr_of_mut!(process_id))) };

    (process_id, thread_id)
  }

  pub fn exe_path(hwnd: HWND) -> Result<String> {
    let mut len = 512_u32;
    let mut path: Vec<u16> = vec![0; len as usize];
    let text_ptr = path.as_mut_ptr();

    let (process_id, _) = Self::window_thread_process_id(hwnd);
    let handle = Self::process_handle(process_id)?;
    unsafe {
      QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(text_ptr), &mut len)
        .expect("Failed to query full process image name");
    }
    Self::close_handle(handle)?;

    Ok(String::from_utf16(&path[..len as usize])?)
  }

  pub fn is_window_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd) }.into()
  }

  pub fn get_window_text(hwnd: HWND) -> String {
    let mut text: [u16; 512] = [0; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut text) };
    let length = usize::try_from(len).unwrap_or(0);
    String::from_utf16(&text[..length]).unwrap_or("".to_owned())
  }

  pub fn is_real_window(hwnd: HWND, ignore_frame: bool) -> bool {
    if !Utils::is_window_visible(hwnd) {
      return false;
    }

    let parent = unsafe { GetParent(hwnd) };
    if parent.0 != 0 {
      return false;
    }

    let ex_style = Utils::get_ex_styles(hwnd);
    let style = Utils::get_styles(hwnd);
    let owner = unsafe { GetWindow(hwnd, GW_OWNER) };
    if !style.contains(WS_VISIBLE) {
      if !ex_style.contains(WS_EX_APPWINDOW)
        || (owner.0 != 0 && ex_style.contains(WS_EX_TOOLWINDOW))
      {
        println!("{}", Utils::get_window_text(hwnd));
        return false;
      }
    }

    if (ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_NOACTIVATE))
      && !ex_style.contains(WS_EX_APPWINDOW)
    {
      return false;
    }

    let exe_path = Utils::exe_path(hwnd).unwrap_or_default();
    if exe_path.starts_with("C:\\Windows\\SystemApps")
      || (!ignore_frame && exe_path.ends_with("ApplicationFrameHost.exe"))
    {
      return false;
    }

    let title = Utils::get_window_text(hwnd);
    if title.is_empty() {
      return false;
    }

    true
  }
}
