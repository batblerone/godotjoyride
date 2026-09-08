use crate::game_manager::*;
use godot::classes::{AnimatedSprite2D, Area2D, CharacterBody2D, ICharacterBody2D, Input};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    #[export]
    climb_speed: f32,
    #[export]
    acceleration: f32,
    animation_player: Option<Gd<AnimatedSprite2D>>, // Stores a reference to the animation node
    pub is_dead: bool,
    was_flying: bool,

    base: Base<CharacterBody2D>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Walk,
    Fly,
    Fall,
    Land,
    Die,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            climb_speed: -600.0,
            acceleration: 1000.0,
            animation_player: None,
            is_dead: false,
            was_flying: false,
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

        self.is_dead = false;
        self.was_flying = false;
        self.base_mut().set_velocity(Vector2::ZERO);
    }

    fn physics_process(&mut self, delta: f64) {
        let input = Input::singleton();
        let is_flying_pressed = input.is_action_pressed("Up");
        let was_flying = self.was_flying;

        // let is_on_floor = self.base().is_on_floor();
        let mut velocity = self.base().get_velocity();

        if is_flying_pressed {
            let weight = (self.acceleration * delta as f32) / self.climb_speed.abs();
            velocity.y = velocity.y + (self.climb_speed - velocity.y) * weight;
        } else {
            velocity.y += self.base().get_gravity().y * delta as f32
        }

        let is_on_floor = self.base().is_on_floor();

        let animation_current = if self.is_dead {
            AnimationState::Die
        } else if is_flying_pressed {
            AnimationState::Fly
        } else if !is_on_floor {
            AnimationState::Fall
        } else if was_flying && is_on_floor {
            AnimationState::Land
        } else {
            AnimationState::Walk
        };

        {
            let mut base = self.base_mut();
            base.set_velocity(velocity);
            base.move_and_slide();
        }

        let animation_name = match animation_current {
            AnimationState::Die => "die",
            AnimationState::Fly => "fly",
            AnimationState::Fall => "fall",
            AnimationState::Land => "land",
            AnimationState::Walk => "walk",
        };

        let mut animate_sprite = self
            .base_mut()
            .get_node_as::<AnimatedSprite2D>("AnimatedSprite2D");
        animate_sprite.play_ex().name(animation_name).done();

        self.was_flying = is_flying_pressed;

        let base = self.base_mut();
        // Loop through everything the character physically collided with during this frame
        for i in 0..base.get_slide_collision_count() {
            if let Some(collision) = base.get_slide_collision(i) {
                if let Some(collider) = collision.get_collider() {
                    // Check if the solid obstacle is a CharacterBody2D
                    if let Ok(obstacle_body) = collider.try_cast::<CharacterBody2D>() {
                        godot_print!(
                            "Collided with a solid CharacterBody2D obstacle: {:?}",
                            obstacle_body.get_name()
                        );
                        // Drop mutable borrow of base to safely call self methods
                        drop(base);
                        self.player_died();
                        return;
                    }
                }
            }
        }
    }
}

#[godot_api]
impl Player {
    #[func]
    fn player_died(&mut self) {
        if self.is_dead {
            return;
        }

        let Some(mut game_manager_node) = self
            .base()
            .try_get_node_as::<GameManager>("/root/GameManagerGlobal")
        else {
            godot_error!("GameManagerGlobal not found in Autoload.");
            return;
        };

        self.is_dead = true;

        let mut game_manager = game_manager_node.bind_mut();
        game_manager.game_over();
    }

    // A method for killing player when body is entered
    #[func]
    fn on_obstacles_entered(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("Player") {
            self.player_died();
        }
    }
}
