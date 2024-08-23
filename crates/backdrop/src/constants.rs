#![allow(dead_code)]

#[repr(C)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
pub struct ACCENT_POLICY {
  // Determines how a window's background is rendered
  pub nAccentState: i32,   // Background effect
  pub nFlags: i32,         // Flags, set to 2 to tell GradientColor is used, rest is unknown
  pub nGradientColor: i32, // Background color
  pub nAnimationId: i32,   // Unknown
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct WINDOWCOMPOSITIONATTRIBDATA {
  // Options for [Get/Set]WindowCompositionAttribute.
  pub Attrib: i32,                   // Type of what is being get or set.
  pub pvData: *mut std::ffi::c_void, // Pointer to memory that will receive what is get or that contains what will be set.
  pub cbData: usize,                 // Size of the data being pointed to by pvData.
}

// Determines what attribute is being manipulated.
pub const WCA_ACCENT_POLICY: i32 = 19; // The attribute being get or set is an accent policy.

// Affects the rendering of the background of a window.
pub const ACCENT_DISABLED: i32 = 0; // Default value, background is black
pub const ACCENT_ENABLE_GRADIENT: i32 = 1; // Background is GradientColor, alpha channel ignored
pub const ACCENT_ENABLE_TRANSPARENTGRADIENT: i32 = 2; // Background is GradientColor
pub const ACCENT_ENABLE_BLURBEHIND: i32 = 3; // Background is GradientColor, with blur effect
pub const ACCENT_ENABLE_ACRYLICBLURBEHIND: i32 = 4; // Background is GradientColor, with acrylic blur effect
pub const ACCENT_ENABLE_HOSTBACKDROP: i32 = 5; // Unknown
pub const ACCENT_INVALID_STATE: i32 = 6; // Unknownn, seems to draw background fully transparent
