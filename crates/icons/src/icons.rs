mod convert;

use image::ImageFormat;
use image::RgbaImage;

use regex::Regex;
use util::AppError;
use walkdir::WalkDir;
use widestring::U16CString;

use windows::core::PCWSTR;
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
use windows::Win32::UI::WindowsAndMessaging::HICON;

use xml::reader::XmlEvent;
use xml::EventReader;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;

use convert::hicon_to_rgba;
use util::Result as AppResult;

pub fn get_images_from_exe(executable_path: &str) -> AppResult<Vec<RgbaImage>> {
  unsafe {
    let path_cstr =
      U16CString::from_str(executable_path).map_err(|_| AppError::from("Invalid path"))?;
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
      .map(hicon_to_rgba)
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

pub fn decode_uri(s: impl AsRef<str>) -> String {
  let re = Regex::new(r"%([A-Fa-f0-9]{2})").unwrap();
  re.replace_all(s.as_ref(), |caps: &regex::Captures| {
    let hex = &caps[1];
    let byte = u8::from_str_radix(hex, 16).unwrap_or(0);
    (byte as char).to_string()
  })
  .into_owned()
}

pub fn get_icon(exe_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
  let images = get_images_from_exe(exe_path);
  let mut png_bytes: Vec<u8> = Vec::new();

  let uwp_icon = get_uwp_icon(exe_path);
  if let Ok(uwp_icon) = uwp_icon {
    png_bytes = uwp_icon;
  }

  if png_bytes.is_empty() {
    if let Ok(images) = images {
      if let Some(icon) = images.first() {
        let mut cursor = Cursor::new(Vec::new());
        if let Err(e) = icon.write_to(&mut cursor, ImageFormat::Png) {
          eprintln!("Failed to write icon to PNG format: {}", e);
        } else {
          png_bytes = cursor.into_inner();
        }
      }
    }
  }

  Ok(png_bytes)
}

fn get_uwp_icon(exe_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
  let dir = Path::new(&exe_path)
    .parent()
    .ok_or("Failed to get parent directory")?;
  let manifest_path = dir.join("AppxManifest.xml");

  if !manifest_path.exists() {
    return Err("Manifest file does not exist".into());
  }

  let file = File::open(&manifest_path)?;
  let file = BufReader::new(file);
  let parser = EventReader::new(file);

  let mut path_to_logo = None;
  let mut inside_properties = false;

  for e in parser {
    match e? {
      XmlEvent::StartElement { name, .. } => {
        if name.local_name == "Properties" {
          inside_properties = true;
        }
        if inside_properties && name.local_name == "Logo" {
          path_to_logo = Some(String::new());
        }
      }
      XmlEvent::Characters(s) => {
        if inside_properties {
          if let Some(ref mut logo) = path_to_logo {
            *logo = s;
          }
        }
      }
      XmlEvent::EndElement { name } => {
        if name.local_name == "Properties" {
          inside_properties = false;
        }
      }
      _ => {}
    }
  }

  let path_to_logo = path_to_logo.ok_or("Logo path not found in manifest")?;
  let logo_path = dir.join(&path_to_logo);

  // Check for .ico file in the logo_path parent directory
  let parent_dir = logo_path.parent().ok_or("Failed to get logo directory")?;
  for entry in WalkDir::new(parent_dir) {
    let entry = entry?;
    if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "ico") {
      let mut file = File::open(entry.path()).unwrap();
      let mut buf = Vec::new();
      file.read_to_end(&mut buf).unwrap();
      return Ok(buf);
    }
  }

  let parent_dir = logo_path.parent().ok_or("Failed to get logo directory")?;
  let file_name_without_extension = logo_path
    .file_stem()
    .ok_or("Failed to get logo file stem")?
    .to_string_lossy()
    .to_string();
  let file_extension = logo_path
    .extension()
    .ok_or("Failed to get logo file extension")?
    .to_string_lossy()
    .to_string();

  let mut final_logo = None;

  for entry in WalkDir::new(parent_dir) {
    let entry = entry?;
    if entry.file_type().is_file() {
      let path = entry.path();
      if let Some(stem) = path.file_stem() {
        if stem
          .to_string_lossy()
          .starts_with(&file_name_without_extension)
          && path.extension().map_or(false, |ext| {
            ext.to_str().unwrap().to_string() == file_extension
          })
        {
          final_logo = Some(path.to_path_buf());
          break;
        }
      }
    }
  }

  let final_logo = final_logo.ok_or("Logo file not found")?;

  let mut file = File::open(&final_logo).unwrap();
  let mut buf = Vec::new();
  file.read_to_end(&mut buf).unwrap();
  Ok(buf)
}

// pub fn get_icon_alternative(hwnd: HWND) -> Vec<u8> {
//   let mut icon = Vec::new();
//   if let Some(hicon) = get_window_icon(hwnd, WPARAM(ICON_BIG as usize)) {
//     let bitmap = convert_hicon_to_rgba_image(&hicon);
//     icon = bitmap.unwrap().to_vec();
//   }

//   if icon.is_empty() {
//     if let Some(hicon) = get_window_icon(hwnd, WPARAM(ICON_SMALL as usize)) {
//       let bitmap = convert_hicon_to_rgba_image(&hicon);
//       icon = bitmap.unwrap().to_vec();
//     }
//   }

//   if icon.is_empty() {
//     if let Some(hicon) = get_window_icon(hwnd, WPARAM(ICON_SMALL2 as usize)) {
//       let bitmap = convert_hicon_to_rgba_image(&hicon);
//       icon = bitmap.unwrap().to_vec();
//     }
//   }

//   if icon.is_empty() {
//     if let Some(hicon) = get_window_class_icon(hwnd, GCLP_HICON.0) {
//       let bitmap = convert_hicon_to_rgba_image(&hicon);
//       icon = bitmap.unwrap().to_vec();
//     }
//   }

//   icon
// }

// fn get_window_icon(hwnd: HWND, icon_type: WPARAM) -> Option<HICON> {
//   let hicon = unsafe { SendMessageW(hwnd, WM_GETICON, icon_type, LPARAM(0)) };
//   if hicon.0 != 0 {
//     Some(HICON(hicon.0 as isize))
//   } else {
//     None
//   }
// }

// fn get_window_class_icon(hwnd: HWND, icon_type: i32) -> Option<HICON> {
//   let hicon = unsafe { GetClassLongPtrW(hwnd, GET_CLASS_LONG_INDEX(icon_type)) };
//   if hicon != 0 {
//     Some(HICON(hicon as isize))
//   } else {
//     None
//   }
// }
