fn main() {
  assert!(
    cfg!(windows),
    "This program must be ran in a Windows system"
  );

  tauri_build::build()
}
