mod dock;
mod hitbox;
mod menubar;

pub fn init() {
  dock::init();
  hitbox::init();
  menubar::init();
}

pub fn kill() {
  unsafe { menubar::remove() }
}
