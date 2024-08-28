pub fn hex_to_rgba_int(hex: String) -> Option<u32> {
  if hex.len() == 9 && hex.starts_with('#') {
    let r = &hex[1..3];
    let g = &hex[3..5];
    let b = &hex[5..7];
    let a = &hex[7..9];
    Some(u32::from_str_radix(&format!("{}{}{}{}", a, b, g, r), 16).ok()?)
  } else {
    Some(0)
  }
}
