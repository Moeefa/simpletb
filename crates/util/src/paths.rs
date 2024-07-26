use std::{env, path::PathBuf};

pub fn home_dir() -> Option<PathBuf> {
  return env::var_os("HOME")
    .and_then(|h| if h.is_empty() { None } else { Some(h) })
    .map(PathBuf::from);
}