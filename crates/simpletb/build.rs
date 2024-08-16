#[cfg(not(target_os = "windows"))]
use log::warn;
use simple_logger::SimpleLogger;

fn main() {
  SimpleLogger::new().init().unwrap();

  #[cfg(not(target_os = "windows"))]
  {
    warn!("This program must be ran in a Windows system");
    std::process::exit(1);
  }

  tauri_build::build()
}
