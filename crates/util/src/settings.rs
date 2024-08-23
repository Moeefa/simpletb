use serde::Deserialize;
use std::{fs, sync::LazyLock};

use crate::home_dir;

#[derive(Deserialize)]
pub struct Settings {
  pub height: i32,
  pub margin_bottom: i32,
}

pub static USER_SETTINGS: LazyLock<Settings> =
  LazyLock::new(|| Settings::load_settings().unwrap_or_default());

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
    Settings::load_settings().unwrap_or_default()
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
