mod constants;
mod hex_converter;

use std::mem;

use constants::ACCENT_ENABLE_ACRYLICBLURBEHIND;
use constants::ACCENT_POLICY;
use constants::WCA_ACCENT_POLICY;

use constants::WINDOWCOMPOSITIONATTRIBDATA;
use windows::core::PCSTR;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;
use windows::Win32::Graphics::Dwm::DWMSBT_MAINWINDOW;
use windows::Win32::Graphics::Dwm::DWMWA_SYSTEMBACKDROP_TYPE;
use windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE;
use windows::Win32::System::LibraryLoader::GetProcAddress;
use windows::Win32::System::LibraryLoader::LoadLibraryA;

use crate::hex_converter::hex_to_rgba_int;

type SetWindowCompositionAttributeFn =
  unsafe extern "system" fn(HWND, *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;

// Auto: DWMSBT_AUTO
// None: DWMSBT_NONE
// Default: DWMSBT_MAINWINDOW
// Mica Alt: DWMSBT_TABBEDWINDOW
// Acrylic: DWMSBT_TRANSIENTWINDOW
//
// https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type

pub fn enable_blur(hwnd: HWND, hex: &str, always_active: bool) {
  if !always_active {
    unsafe {
      // Set system backdrop
      DwmSetWindowAttribute(
        hwnd,
        DWMWA_SYSTEMBACKDROP_TYPE,
        &DWMSBT_MAINWINDOW as *const _ as _,
        std::mem::size_of::<DWMWINDOWATTRIBUTE>() as _,
      )
      .expect("Failed to set window attribute");
    }
  } else {
    // Always active backdrop

    // Accent policy
    let accent = ACCENT_POLICY {
      nAccentState: ACCENT_ENABLE_ACRYLICBLURBEHIND,
      nFlags: 2,
      nGradientColor: hex_to_rgba_int(hex).unwrap() as i32,
      nAnimationId: 0,
    };

    // Window composition attribute data
    let mut data = WINDOWCOMPOSITIONATTRIBDATA {
      Attrib: WCA_ACCENT_POLICY,
      pvData: &accent as *const _ as *mut _,
      cbData: mem::size_of::<ACCENT_POLICY>(),
    };

    #[allow(non_snake_case)]
    unsafe {
      // Load user32.dll
      let hmodule = LoadLibraryA(PCSTR("user32.dll\0".as_ptr() as *const u8))
        .expect("Failed to load user32.dll");

      // Get SetWindowCompositionAttribute address
      let SetWindowCompositionAttribute: SetWindowCompositionAttributeFn = std::mem::transmute(
        GetProcAddress(
          hmodule,
          PCSTR("SetWindowCompositionAttribute\0".as_ptr() as *const u8),
        )
        .expect("Failed to get SetWindowCompositionAttribute address"),
      );

      // Set window composition attribute
      SetWindowCompositionAttribute(hwnd, &mut data)
        .expect("Failed to set window composition attribute");
    }
  }
}
