// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_bar;
mod dock;
mod hitbox;
mod utils;

use crate::app_bar::*;
use crate::utils::blur_window::enable_blur;
use crate::utils::error_handler::Result;
use crate::utils::icons_to_buff::get_icon;
use crate::utils::Utils;

use dock::MARGIN_BOTTOM;
use lazy_static::lazy_static;

use tauri::window::{Effect, EffectsBuilder};
use tauri::{Manager, WebviewWindow};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};

use std::fs;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;

use active_win_pos_rs::{get_active_window, ActiveWindow};
use ctrlc;
use regex::Regex;

use windows::core::PCSTR;
use windows::core::PSTR;
use windows::Win32::Foundation::{CloseHandle, FALSE, HWND, TRUE};
use windows::Win32::System::Threading::{
  CreateProcessA, WaitForInputIdle, CREATE_NEW_CONSOLE, INFINITE, PROCESS_INFORMATION, STARTUPINFOA,
};
use windows::Win32::UI::WindowsAndMessaging::{
  GetForegroundWindow, GetWindowPlacement, MoveWindow, SetForegroundWindow, SetMenu,
  SetWindowLongPtrA, ShowWindow, EVENT_MAX, EVENT_MIN, EVENT_OBJECT_FOCUS, EVENT_SYSTEM_FOREGROUND,
  GWL_EXSTYLE, SW_MINIMIZE, SW_RESTORE, WINDOWPLACEMENT, WINEVENT_OUTOFCONTEXT,
  WINEVENT_SKIPOWNPROCESS, WS_EX_TOOLWINDOW,
};

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
  buffer: Vec<u8>,
  hwnd: isize,
}

type SetFocusFn = unsafe extern "system" fn(HWND) -> HWND;

lazy_static! {
  static ref WINDOW: Mutex<Option<tauri::WebviewWindow>> = Mutex::new(None);
  static ref PREV_WINDOW: Mutex<ActiveWindow> = Mutex::new(ActiveWindow::default());
  static ref IS_FULSCREEN: Mutex<bool> = Mutex::new(false);
}

#[tauri::command]
fn show_window(hwnd: isize) {
  unsafe {
    let hmodule =
      LoadLibraryA(PCSTR("user32.dll\0".as_ptr() as *const u8)).expect("Failed to load user32.dll");

    #[allow(non_snake_case)]
    let SetFocus: SetFocusFn = std::mem::transmute(
      GetProcAddress(hmodule, PCSTR("SetFocus\0".as_ptr() as *const u8))
        .expect("Failed to get SetFocus address"),
    );

    let hwnd: HWND = std::mem::transmute(hwnd);
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
async fn open_settings(app: tauri::AppHandle) {
  // Create a settings window
  // ...

  let window = Some(app.get_webview_window("settings")).unwrap_or(None);
  if window == None {
    let _ = tauri::WebviewWindowBuilder::new(
      &app,
      "settings",
      tauri::WebviewUrl::App("/src/settings/index.html".into()),
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

#[allow(unused_variables)]
#[tauri::command]
async fn open_context(app: tauri::AppHandle, x: i32, y: i32) {
  // Create a settings window
  // ...
  let window = Some(app.get_webview_window("context")).unwrap_or(None);
  if window != None {
    window.unwrap().destroy().unwrap();
  }

  let screen = unsafe { ScreenGeometry::new() };

  let window = tauri::WebviewWindowBuilder::new(
    &app,
    "context",
    tauri::WebviewUrl::App("/src/settings/index.html".into()),
  )
  .title("Context")
  .resizable(false)
  .inner_size(220.0, 120.0)
  .transparent(true)
  .always_on_top(true)
  .effects(EffectsBuilder::new().effects([Effect::Mica]).build())
  .build()
  .expect("Failed to create settings window");

  unsafe {
    let hwnd: HWND = std::mem::transmute(window.hwnd().unwrap().0);
    thread::spawn(move || {
      MoveWindow(
        hwnd,
        (screen.width / 2) - (220 / 2),
        screen.height - 51 - MARGIN_BOTTOM - 120,
        220,
        120,
        true,
      )
      .unwrap();
    });
  }
}

#[tauri::command]
fn execute(commandline: String, mut applicationname: String) {
  unsafe {
    if applicationname.contains("%USERPROFILE%") {
      let user_profile = env::var("HOME").unwrap_or("".to_string());
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
      FALSE,
      CREATE_NEW_CONSOLE,
      None,
      None,
      &mut startup_info,
      &mut process_info,
    )
    .is_ok()
    {
      WaitForInputIdle(process_info.hProcess, INFINITE);
      CloseHandle(process_info.hProcess).expect("Failed to close process");
      CloseHandle(process_info.hThread).expect("Failed to close thread");
    } else {
      println!("Failed to create process");
    }
  }
}

fn main() {
  // Initialize Tokio runtime
  let runtime = tokio::runtime::Runtime::new().unwrap();

  // Load configuration from file
  let config = load_configuration().expect("Failed to load configuration");

  // Initialize Tauri application
  let app_builder = initialize_tauri_app();

  // Setup Tauri application
  let app_builder = setup_tauri_app(app_builder, config);

  // Run Tauri application within the Tokio runtime
  runtime.block_on(async {
    if let Err(err) = app_builder.run(tauri::generate_context!()) {
      eprintln!("Error while running Tauri application: {:?}", err);
    }
  });
}

// Function to load configuration from file
fn load_configuration() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
  let file = fs::File::open(format!("{}\\.simpletb\\config.json", env::var("HOME").unwrap_or("".to_string())))?;
  let config: serde_json::Value = serde_json::from_reader(file)?;
  Ok(config)
}

// Function to initialize Tauri application
fn initialize_tauri_app() -> tauri::Builder<tauri::Wry> {
  tauri::Builder::default()
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![
      execute,
      show_window,
      open_settings,
      open_context
    ])
}

// Function to setup Tauri application
fn setup_tauri_app(
  app_builder: tauri::Builder<tauri::Wry>,
  config: serde_json::Value,
) -> tauri::Builder<tauri::Wry> {
  return app_builder.setup(move |app| {
    let window = app.get_webview_window("main").unwrap();
    let hwnd = unsafe { std::mem::transmute(window.hwnd().unwrap().0) };
    let app_bar = Arc::new(Mutex::new(AppBar::new(&window)));
    let app_bar_clone = app_bar.clone();

    let round_window = create_round_window(app.app_handle().clone(), config.clone()).unwrap();

    let window_clone = window.clone();
    let round_window_clone = round_window.clone();
    app.listen("app-not-fullscreen", move |_msg| {
      if !window_clone.is_visible().unwrap() {
        window_clone.show().unwrap();
        round_window_clone.show().unwrap();
      }
    });

    let window_clone = window.clone();
    let round_window_clone = round_window.clone();
    app.listen("app-fullscreen", move |_msg| {
      if window_clone.is_visible().unwrap() {
        window_clone.hide().unwrap();
        round_window_clone.hide().unwrap();
      };
    });

    hitbox::Hitbox::new(app.app_handle().clone())
      .init()
      .expect("Failed to initialize hitbox");

    let dock = dock::Dock::new(app.app_handle().clone());
    dock.init().expect("Failed to initialize dock");

    let dock_clone = dock.clone();
    app.listen("mouse-in", move |_msg| {
      unsafe { dock_clone.show() };
    });

    let dock_clone = dock.clone();
    app.listen("mouse-out", move |_msg| {
      unsafe { dock_clone.hide() };
    });

    // Set up menu event handler (only for desktop)
    #[cfg(desktop)]
    {
      app.on_menu_event(move |_app_handle: &tauri::AppHandle, event| {
        println!("context menu clicked!");
        println!("menu event: {:?}", event);
        // Access app_bar as needed using app_bar.lock().unwrap()
      });
    }

    // Set up window event handlers
    let app_bar = Arc::clone(&app_bar);
    window.on_window_event(move |event| {
      let app_bar = app_bar.lock().unwrap();
      match event {
        tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed {} => {
          println!("Close requested");
          unsafe {
            app_bar.remove_app_bar();
            Utils::hide_taskbar(false);
          };
        }
        _ => {}
      }
    });

    // Enable blur effect if configured
    let config_clone = config.clone();
    let hwnd_clone = hwnd;
    tokio::spawn(async move {
      if let Some(blur_enabled) = config_clone.get("blur").and_then(|v| v.as_bool()) {
        if blur_enabled {
          let hex = config_clone
            .get("hex")
            .and_then(|v| v.as_str())
            .unwrap_or("#0000004d");
          let blur_always_active = config_clone
            .get("blurAlwaysActive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
          enable_blur(hwnd_clone, hex, blur_always_active).expect("Failed to enable blur");
        }
      }
    });

    // Set window style
    unsafe {
      app_bar_clone
        .lock()
        .unwrap()
        .add_app_bar()
        .expect("Failed to add app bar");

      SetWindowLongPtrA(hwnd, GWL_EXSTYLE, WS_EX_TOOLWINDOW.0 as isize);

      // Set window size using MoveWindow
      let width = ScreenGeometry::new().width;
      let height = config.get("height").and_then(|v| v.as_i64()).unwrap_or(32) as i32;
      MoveWindow(hwnd, 0, 0, width, height, TRUE).expect("Failed to move window");
    }

    let window_clone = window.clone();
    *WINDOW.lock().unwrap() = Some(window.clone());

    // Spawn asynchronous tasks for monitoring active window and configuration file changes
    spawn_window_monitor_thread();
    spawn_config_file_monitor_thread();

    // Set Ctrl-C handler
    set_ctrlc_handler(app_bar_clone, window_clone);

    Ok(())
  });
}

fn create_round_window(
  app: tauri::AppHandle,
  config: serde_json::Value,
) -> Result<WebviewWindow, String> {
  // Create a round window
  // ...
  let window_height = config.get("height").and_then(|v| v.as_i64()).unwrap_or(32) as f64;
  let (width, _height) = unsafe { (ScreenGeometry::new().width, ScreenGeometry::new().height) };

  let webview_window = tauri::WebviewWindowBuilder::new(
    &app,
    "round-border",
    tauri::WebviewUrl::App("/src/round-border/index.html".into()),
  )
  .title("rounded")
  .decorations(false)
  .resizable(false)
  .transparent(true)
  .inner_size(width.into(), 20.0)
  .position(0.0, window_height)
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
    MoveWindow(hwnd, 0, window_height as i32, width, 20, TRUE).expect("Failed to move window");
  }

  Ok(webview_window)
}

// Function to spawn thread for monitoring active window
fn spawn_window_monitor_thread() {
  unsafe extern "system" fn win_event_hook_callback(
    _hook_handle: HWINEVENTHOOK,
    _event_id: u32,
    _window_handle: HWND,
    _object_id: i32,
    _child_id: i32,
    _thread_id: u32,
    _timestamp: u32,
  ) {
    match _event_id {
      EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND => match get_active_window() {
        Ok(active_window) => {
          #[allow(unused_variables)]
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
            && !*IS_FULSCREEN.lock().unwrap()
          {
            WINDOW
              .lock()
              .unwrap()
              .as_ref()
              .unwrap()
              .emit("app-fullscreen", ())
              .expect("Failed to emit app fullscreen");
          } else {
            WINDOW
              .lock()
              .unwrap()
              .as_ref()
              .unwrap()
              .emit("app-not-fullscreen", ())
              .expect("Failed to emit app fullscreen");
          }

          if active_window.app_name != PREV_WINDOW.lock().unwrap().app_name.as_str()
            && active_window.app_name != env!("CARGO_PKG_DESCRIPTION")
          {
            let icon = get_icon(&active_window.process_path.to_str().ok_or("").unwrap(), 24);

            *PREV_WINDOW.lock().unwrap() = active_window.clone();
            WINDOW
              .lock()
              .unwrap()
              .as_ref()
              .unwrap()
              .emit(
                "active-window",
                Payload {
                  message: active_window.app_name,
                  buffer: icon.expect("Failed to get icon"),
                  hwnd: active_window_hwnd,
                },
              )
              .expect("Failed to emit active window");
          }
        }
        Err(_err) => {
          *PREV_WINDOW.lock().unwrap() = ActiveWindow::default();
          WINDOW
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .emit_to(
              "main",
              "active-window",
              Payload {
                message: "Windows Explorer".to_owned(),
                buffer: Vec::new(),
                hwnd: -1,
              },
            )
            .expect("Failed to emit default active window");
        }
      },
      _ => {}
    }
  }

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

// Function to spawn thread for monitoring configuration file changes
fn spawn_config_file_monitor_thread() {
  thread::spawn(move || {
    // Implementation of configuration file monitoring logic
    // ...
  });
}

// Function to set Ctrl-C handler
fn set_ctrlc_handler(app_bar: Arc<Mutex<AppBar>>, window: tauri::WebviewWindow) {
  ctrlc::set_handler(move || {
    unsafe {
      app_bar.lock().unwrap().remove_app_bar();
      Utils::hide_taskbar(false);
    };
    window.destroy().unwrap();
  })
  .expect("Error setting Ctrl-C handler");
}
