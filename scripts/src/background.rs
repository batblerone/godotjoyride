use godot::classes::{IParallax2D, Parallax2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Parallax2D)]
pub struct Background {
    #[export]
    scroll_speed: f32,
    base: Base<Parallax2D>,
}

#[godot_api]
impl IParallax2D for Background {
    fn init(base: Base<Parallax2D>) -> Self {
        Self {
            scroll_speed: -300.0,
            base,
        }
    }

    fn ready(&mut self) {
        let background_scroll = Vector2::new(self.scroll_speed, 0.0);
        let mut background = self.base_mut();
        background.set_ignore_camera_scroll(true);
        background.set_autoscroll(background_scroll);
    }
}
