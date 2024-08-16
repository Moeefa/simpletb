fn main() {
  #[cfg(not(target_os = "windows"))]
  {
    println!("This program must be ran in a Windows system");
    std::process::exit(1);
  }

  tauri_build::build()
}
