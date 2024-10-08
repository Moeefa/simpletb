#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod hooks;

use commands::*;
use util::APP_HANDLE;

fn main() {
  // Initialize Tokio runtime
  let runtime = tokio::runtime::Runtime::new().unwrap();

  // Initialize Tauri application
  let app_builder = initialize_tauri_app();

  // Setup Tauri application
  let app_builder = setup_tauri_app(app_builder);

  // Run Tauri application within the Tokio runtime
  runtime.block_on(async {
    if let Err(err) = app_builder.run(tauri::generate_context!()) {
      eprintln!("Error while running Tauri application: {:?}", err);
    }
  });
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
fn setup_tauri_app(app_builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
  app_builder
    .setup(move |app| {
      // Initialize app handle
      *APP_HANDLE.lock().unwrap() = Some(app.handle().clone());

      hooks::init();
      ui::init();

      Ok(())
    })
    .on_window_event(|_window, event| match event {
      tauri::WindowEvent::Destroyed | tauri::WindowEvent::CloseRequested { .. } => ui::kill(),
      _ => {}
    })
}
