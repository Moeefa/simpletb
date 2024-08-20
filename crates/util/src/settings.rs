use lazy_static::lazy_static;
use serde::Deserialize;
use std::fs;

use crate::home_dir;

#[derive(Deserialize)]
pub struct Settings {
  pub height: i32,
  pub margin_bottom: i32,
}

lazy_static! {
  pub static ref USER_SETTINGS: Settings = Settings::new();
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      height: 26,
      margin_bottom: 5,
    }
  }
}

impl Settings {
  pub fn new() -> Self {
    let settings = Settings::load_settings().unwrap_or_default();
    settings
  }

  pub fn load_settings() -> Result<Self, Box<dyn std::error::Error>> {
    let file = fs::File::open(format!(
      "{}\\.simpletb\\config.json",
      home_dir().unwrap_or_default().display()
    ))?;

    let config: Settings = serde_json::from_reader(file)?;
    Ok(config)
  }
}
