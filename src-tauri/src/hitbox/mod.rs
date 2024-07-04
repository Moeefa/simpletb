use crate::ScreenGeometry;
use tauri::Manager;
use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::MoveWindow,
};

pub struct Hitbox {
  handle: tauri::AppHandle,
}

impl Hitbox {
  pub fn new(handle: tauri::AppHandle) -> Self {
    Self { handle }
  }

  pub fn setup_window(&self) -> Result<tauri::WebviewWindow, ()> {
    let window = tauri::WebviewWindowBuilder::new(
      &self.handle,
      "hitbox",
      tauri::WebviewUrl::App("/src/hitbox/index.html".into()),
    )
    .title("dock")
    .transparent(true)
    .always_on_top(true)
    .decorations(false)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .shadow(false)
    .skip_taskbar(true)
    .build()
    .expect("Failed to build dock window");

    Ok(window)
  }

  pub fn init(&self) -> Result<(), &'static str> {
    let window = self.setup_window().expect("Failed to setup dock window");
    let hwnd: HWND = unsafe { std::mem::transmute(window.hwnd().unwrap().0) };

    unsafe {
      // let pci = std::ptr::null_mut();
      // GetCursorInfo(pci).unwrap();

      // if (*pci).flags.0 & CURSOR_SHOWING.0 != 0 {
      //   println!("Mouse is in the window");
      // }

      let screen_rect = ScreenGeometry::new();
      // SetWindowPos(hwnd, HWND_TOPMOST, 0, screen_rect.height, screen_rect.width, 2, SWP_NOSIZE | SWP_NOMOVE).expect("Failed to set window position");
      MoveWindow(hwnd, 0, screen_rect.height - 2, screen_rect.width, 2, true)
        .expect("Failed to move window");
    }

    let handle_clone = self.handle.clone();
    window.listen("mouse-in", move |_msg| {
      handle_clone
        .emit("hide-taskbar", ())
        .expect("Failed to emit hide-taskbar event");
    });

    Ok(())
  }
}
