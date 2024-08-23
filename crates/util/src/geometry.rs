use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

pub struct ScreenGeometry {
  pub x: i32,
  pub y: i32,
  pub height: i32,
  pub width: i32,
}

impl Default for ScreenGeometry {
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      height: 0,
      width: 0,
    }
  }
}

impl ScreenGeometry {
  pub fn new() -> Self {
    let mut screen_rect = RECT::default();
    let mut geometry: ScreenGeometry = Self::default();

    unsafe { GetWindowRect(GetDesktopWindow(), &mut screen_rect).unwrap() };
    geometry.x = screen_rect.left;
    geometry.y = screen_rect.top;
    geometry.width = screen_rect.right;
    geometry.height = screen_rect.bottom;

    geometry
  }
}
