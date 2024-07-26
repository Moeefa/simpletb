use std::{env, fs};
use lazy_static::lazy_static;

lazy_static! {
  pub static ref USER_SETTINGS: serde_json::Value = load_settings().expect("Failed to load configuration");
}

pub fn load_settings() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
  let file = fs::File::open(format!("{}\\.simpletb\\config.json", env::var("HOME").unwrap_or_default()))?;
  let config: serde_json::Value = serde_json::from_reader(file)?;
  Ok(config)
}

