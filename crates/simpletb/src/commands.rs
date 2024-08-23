use std::{env, path::PathBuf, ptr::null_mut, thread};

use tauri::{
  window::{Effect, EffectsBuilder},
  Manager,
};
use util::{ScreenGeometry, USER_SETTINGS};

use windows::core::PCSTR;
use windows::core::PSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::CreateProcessA;
use windows::Win32::System::Threading::WaitForInputIdle;
use windows::Win32::System::Threading::CREATE_NEW_CONSOLE;
use windows::Win32::System::Threading::INFINITE;
use windows::Win32::System::Threading::PROCESS_INFORMATION;
use windows::Win32::System::Threading::STARTUPINFOA;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindowPlacement;
use windows::Win32::UI::WindowsAndMessaging::MoveWindow;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;
use windows::Win32::UI::WindowsAndMessaging::SW_RESTORE;
use windows::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT;

#[tauri::command]
pub fn show_window(hwnd: isize) {
  unsafe {
    let hwnd: HWND = HWND(hwnd);
    let mut placement: WINDOWPLACEMENT = WINDOWPLACEMENT::default();
    GetWindowPlacement(hwnd, &mut placement).expect("Failed to get window placement");
    if placement.showCmd == 2 {
      ShowWindow(hwnd, SW_RESTORE);
      SetForegroundWindow(hwnd).expect("Failed to set foreground window");
      SetFocus(hwnd);
    } else {
      if GetForegroundWindow() == hwnd {
        ShowWindow(hwnd, SW_MINIMIZE);
      } else {
        SetForegroundWindow(hwnd).expect("Failed to set foreground window");
        SetFocus(hwnd);
      }
    }
  }
}

#[tauri::command]
pub async fn open_settings(app: tauri::AppHandle) {
  if app.get_webview_window("settings").is_none() {
    tauri::WebviewWindowBuilder::new(
      &app,
      "settings",
      tauri::WebviewUrl::App("/crates/ui/src/displays/settings/index.html".into()),
    )
    .title("Settings")
    .resizable(false)
    .inner_size(450.0, 600.0)
    .transparent(true)
    .effects(EffectsBuilder::new().effects([Effect::Mica]).build())
    .build()
    .expect("Failed to create settings window");
  }
}

#[tauri::command]
pub async fn open_context(app: tauri::AppHandle, _x: i32, _y: i32) {
  if let Some(window) = app.get_webview_window("context") {
    window.close().unwrap();
  }

  let screen = ScreenGeometry::new();

  let window = tauri::WebviewWindowBuilder::new(
    &app,
    "context",
    tauri::WebviewUrl::App("/crates/ui/src/displays/settings/index.html".into()),
  )
  .title("Context")
  .resizable(false)
  .inner_size(220.0, 120.0)
  .transparent(true)
  .always_on_top(true)
  .effects(EffectsBuilder::new().effects([Effect::Mica]).build())
  .build()
  .expect("Failed to create context window");

  unsafe {
    let hwnd: HWND = HWND(window.hwnd().unwrap().0);
    thread::spawn(move || {
      MoveWindow(
        hwnd,
        (screen.width / 2) - (220 / 2),
        screen.height - 51 - USER_SETTINGS.margin_bottom - 120,
        220,
        120,
        true,
      )
      .unwrap();
    });
  }
}

#[tauri::command]
pub fn execute(commandline: String, mut applicationname: String) {
  unsafe {
    if applicationname.contains("%USERPROFILE%") {
      let user_profile = env::var("HOME").unwrap_or_default();
      applicationname = applicationname.replace("%USERPROFILE%", &user_profile);
      if !PathBuf::from(&applicationname).exists() {
        println!("File does not exist: {}", applicationname);
        return;
      }
    }

    println!("Executing: {} {}", applicationname, commandline);

    let mut startup_info = STARTUPINFOA::default();
    let mut process_info = PROCESS_INFORMATION::default();

    if CreateProcessA(
      if applicationname.is_empty() {
        PCSTR(null_mut())
      } else {
        PCSTR(applicationname.as_ptr() as *const u8)
      },
      if commandline.is_empty() {
        PSTR(null_mut())
      } else {
        PSTR(commandline.as_ptr() as *mut u8)
      },
      None,
      None,
      false,
      CREATE_NEW_CONSOLE,
      None,
      None,
      &mut startup_info,
      &mut process_info,
    )
    .is_ok()
    {
      WaitForInputIdle(process_info.hProcess, INFINITE);
      CloseHandle(process_info.hProcess).unwrap_or_else(|_| println!("Failed to close process"));
      CloseHandle(process_info.hThread).unwrap_or_else(|_| println!("Failed to close thread"));
    } else {
      println!("Failed to create process");
    }
  }
}
