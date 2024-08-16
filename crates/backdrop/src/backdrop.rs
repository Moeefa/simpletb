mod hex_converter;

use std::mem;

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

#[repr(C)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
struct ACCENT_POLICY {
  nAccentState: i32,
  nFlags: i32,
  nGradientColor: i32,
  nAnimationId: i32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct WINDOWCOMPOSITIONATTRIBDATA {
  Attrib: i32,
  pvData: *mut std::ffi::c_void,
  cbData: usize,
}

const ACCENT_ENABLE_ACRYLICBLURBEHIND: i32 = 4;
const WCA_ACCENT_POLICY: i32 = 19;

type SetWindowCompositionAttributeFn =
  unsafe extern "system" fn(HWND, *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;

pub fn enable_blur(hwnd: HWND, hex: &str, always_active: bool) -> Result<(), &'static str> {
  // Set system backdrop
  if !always_active {
    // Auto: DWMSBT_AUTO
    // None: DWMSBT_NONE
    // Default: DWMSBT_MAINWINDOW
    // Mica Alt: DWMSBT_TABBEDWINDOW
    // Acrylic: DWMSBT_TRANSIENTWINDOW
    unsafe {
      DwmSetWindowAttribute(
        hwnd,
        DWMWA_SYSTEMBACKDROP_TYPE,
        &DWMSBT_MAINWINDOW as *const _ as _,
        std::mem::size_of::<DWMWINDOWATTRIBUTE>() as _,
      )
      .expect("Failed to set window attribute");
    }

    return Ok(());
  }

  // Always active backdrop
  let accent = ACCENT_POLICY {
    nAccentState: ACCENT_ENABLE_ACRYLICBLURBEHIND,
    nFlags: 2,
    nGradientColor: hex_to_rgba_int(hex).unwrap() as i32,
    nAnimationId: 0,
  };

  let mut data = WINDOWCOMPOSITIONATTRIBDATA {
    Attrib: WCA_ACCENT_POLICY,
    pvData: &accent as *const _ as *mut _,
    cbData: mem::size_of::<ACCENT_POLICY>(),
  };

  #[allow(non_snake_case)]
  unsafe {
    let hmodule =
      LoadLibraryA(PCSTR("user32.dll\0".as_ptr() as *const u8)).expect("Failed to load user32.dll");

    let SetWindowCompositionAttribute: SetWindowCompositionAttributeFn = std::mem::transmute(
      GetProcAddress(
        hmodule,
        PCSTR("SetWindowCompositionAttribute\0".as_ptr() as *const u8),
      )
      .expect("Failed to get SetWindowCompositionAttribute address"),
    );

    SetWindowCompositionAttribute(hwnd, &mut data)
      .expect("Failed to set window composition attribute");

    Ok(())
  }
}
