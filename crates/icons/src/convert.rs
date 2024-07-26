use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::ICONINFOEXW;
use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
use windows::Win32::Graphics::Gdi::DeleteDC;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::BITMAPINFO;
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::UI::WindowsAndMessaging::GetIconInfoExW;
use image::ImageBuffer;
use image::RgbaImage;
use handler::Result;

use std::arch::x86_64::__m128i;
use std::arch::x86_64::_mm_loadu_si128;
use std::arch::x86_64::_mm_setr_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_shuffle_epi8;
use std::arch::x86_64::_mm_storeu_si128;

/// Convert BGRA to RGBA
///
/// Uses SIMD to go fast
pub fn bgra_to_rgba(data: &mut [u8]) {
  // The shuffle mask for converting BGRA -> RGBA
  let mask: __m128i = unsafe {
      _mm_setr_epi8(
          2, 1, 0, 3, // First pixel
          6, 5, 4, 7, // Second pixel
          10, 9, 8, 11, // Third pixel
          14, 13, 12, 15, // Fourth pixel
      )
  };
  // For each 16-byte chunk in your data
  for chunk in data.chunks_exact_mut(16) {
      let mut vector = unsafe { _mm_loadu_si128(chunk.as_ptr() as *const __m128i) };
      vector = unsafe { _mm_shuffle_epi8(vector, mask) };
      unsafe { _mm_storeu_si128(chunk.as_mut_ptr() as *mut __m128i, vector) };
  }
}

pub fn convert_hicon_to_rgba_image(hicon: &HICON) -> Result<RgbaImage> {
  unsafe {
      let mut icon_info = ICONINFOEXW {
          cbSize: std::mem::size_of::<ICONINFOEXW>() as u32,
          ..Default::default()
      };

      if !GetIconInfoExW(*hicon, &mut icon_info).as_bool() {
          return Err("Failed to get icon info".to_string().into());
      }
      let hdc_screen = CreateCompatibleDC(None);
      let hdc_mem = CreateCompatibleDC(hdc_screen);
      let hbm_old = SelectObject(hdc_mem, icon_info.hbmColor);

      let mut bmp_info = BITMAPINFO {
          bmiHeader: BITMAPINFOHEADER {
              biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
              biWidth: icon_info.xHotspot as i32 * 2,
              biHeight: -(icon_info.yHotspot as i32 * 2),
              biPlanes: 1,
              biBitCount: 32,
              biCompression: DIB_RGB_COLORS.0,
              ..Default::default()
          },
          ..Default::default()
      };

      let mut buffer: Vec<u8> =
          vec![0; (icon_info.xHotspot * 2 * icon_info.yHotspot * 2 * 4) as usize];

      if GetDIBits(
          hdc_mem,
          icon_info.hbmColor,
          0,
          icon_info.yHotspot * 2,
          Some(buffer.as_mut_ptr() as *mut _),
          &mut bmp_info,
          DIB_RGB_COLORS,
      ) == 0
      {
          return Err("Failed to get dibits".into());
      }
      // Clean up
      SelectObject(hdc_mem, hbm_old);
      DeleteDC(hdc_mem).ok()?;
      DeleteDC(hdc_screen).ok()?;
      DeleteObject(icon_info.hbmColor).ok()?;
      DeleteObject(icon_info.hbmMask).ok()?;

      bgra_to_rgba(buffer.as_mut_slice());

      let image = ImageBuffer::from_raw(icon_info.xHotspot * 2, icon_info.yHotspot * 2, buffer)
          .expect("Failed to create image buffer");
      Ok(image)
  }
}
