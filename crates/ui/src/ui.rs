mod dock;
mod hitbox;
mod hooks;
mod menubar;

pub fn init() {
  dock::init();
  hitbox::init();
  hooks::init();
  menubar::init();
}

pub fn kill() {
  menubar::remove();
}
