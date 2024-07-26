mod convert;

use convert::convert_hicon_to_rgba_image;

use handler::AppError;
use image::ImageFormat;
use image::RgbaImage;
use widestring::U16CString;
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
use windows::Win32::UI::WindowsAndMessaging::HICON;

use std::io::Cursor;

use handler::Result;

pub fn get_images_from_exe(executable_path: &str) -> Result<Vec<RgbaImage>> {
    unsafe {
        let path_cstr = U16CString::from_str(executable_path).map_err(|_| AppError::from("Invalid path"))?;
        let path_pcwstr = PCWSTR(path_cstr.as_ptr());
        let num_icons_total = ExtractIconExW(path_pcwstr, -1, None, None, 0);
        if num_icons_total == 0 {
            return Ok(Vec::new()); // No icons extracted
        }

        let mut large_icons = vec![HICON::default(); num_icons_total as usize];
        let mut small_icons = vec![HICON::default(); num_icons_total as usize];
        let num_icons_fetched = ExtractIconExW(
            path_pcwstr,
            0,
            Some(large_icons.as_mut_ptr()),
            Some(small_icons.as_mut_ptr()),
            num_icons_total,
        );

        if num_icons_fetched == 0 {
            return Ok(Vec::new()); // No icons extracted
        }

        let images = large_icons
            .iter()
            .chain(small_icons.iter())
            .map(convert_hicon_to_rgba_image)
            .filter_map(|r| match r {
                Ok(img) => Some(img),
                Err(e) => {
                    println!("Failed to convert HICON to RgbaImage: {:?}", e);
                    None
                }
            })
            .collect();

        large_icons
            .iter()
            .chain(small_icons.iter())
            .filter(|icon| !icon.is_invalid())
            .map(|icon| DestroyIcon(*icon))
            .filter_map(|r| r.err())
            .for_each(|e| {
                println!("Failed to destroy icon: {:?}", e);
            });

        Ok(images)
    }
}

/// returns the path of the icon extracted from the executable or copied if is an UWP app.
///
/// If the icon already exists, it returns the path instead overriding, this is needed for allow user custom icons.
pub fn get_icon(exe_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let images = get_images_from_exe(exe_path);
  let mut png_bytes: Vec<u8> = Vec::new();
  if let Ok(images) = images {
      // Icon on index 0 is always the app's displayed icon
      if let Some(icon) = images.first() {
          let mut cursor = Cursor::new(Vec::new());
          icon.write_to(&mut cursor, ImageFormat::Png)?;
          png_bytes = cursor.into_inner();
      }
  }

  Ok(png_bytes)
}