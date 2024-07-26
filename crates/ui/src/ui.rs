pub mod hitbox;
pub mod dock;
pub mod menubar;

pub fn init_ui() {
  dock::init();
  hitbox::init();
  menubar::init();
}