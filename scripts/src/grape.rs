use crate::player::*;
use godot::classes::{AnimatedSprite2D, Area2D, IArea2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Grapes {
    #[export]
    fly_speed: f32,
    is_visible: bool,
    animation_player: Option<Gd<AnimatedSprite2D>>,

    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Grapes {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            fly_speed: -5.0,
            is_visible: true,
            animation_player: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.animation_player = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite2D");
        if self.animation_player.is_none() {
            godot_print!("Warning: AnimationPlayer node not found at root.");
        }

        self.is_visible = true;
    }
}

#[godot_api]
impl Grapes {
    #[func]
    fn _on_body_entered(&mut self, mut body: Gd<Player>) {
        if body.is_in_group("Player") {
            body.set_visible(false);
        }
    }
}
