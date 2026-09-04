use crate::game_manager::*;
use crate::player::*;
use godot::classes::{AnimatedSprite2D, Area2D, AudioStreamPlayer2D, IArea2D, Node2D, Timer};
use godot::global::randf_range;
use godot::prelude::*;

// This node is for identifying and modifying the grapes
#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Grapes {
    #[export]
    fly_speed: f32,
    is_visible: bool,
    animation_player: Option<Gd<AnimatedSprite2D>>,
    pickup_sound: Option<Gd<AudioStreamPlayer2D>>,

    base: Base<Area2D>,
}

// This node is for actually spawning the grapes
#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct GrapeSpawn {
    #[export]
    spawn_interval: f64,
    #[export]
    grape_scene_path: Option<Gd<PackedScene>>,
    player_node: Option<Gd<Player>>,
    timer: Option<Gd<Timer>>,
    base: Base<Node2D>,
}

// ====================================
// Handle Grapes
// ====================================
#[godot_api]
impl IArea2D for Grapes {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            fly_speed: -5.0,
            is_visible: true,
            animation_player: None,
            pickup_sound: None,
            base,
        }
    }

    fn ready(&mut self) {
        // Find animation
        if let Some(mut animated) = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite2D")
        {
            animated.play(); // Plays default animation
            self.animation_player = Some(animated);
        } else {
            godot_print!("Warning: AnimationPlayer node not found at root.");
        }

        self.pickup_sound = Some(
            self.base()
                .get_node_as::<AudioStreamPlayer2D>("PickupSounds"),
        );

        self.is_visible = true;
    }

    // Moves the grapes by altering position of Grapes Node.
    fn physics_process(&mut self, delta: f64) {
        // Debug for check loop fire
        // godot_print!("Grapes fly_speed: {}", self.fly_speed);

        // Slllliiiiiide to the left (actually move, not slide)
        let movement = Vector2::new(self.fly_speed, 0.0) * delta as f32 * 80.0;
        let mut grape = self.base_mut();
        grape.translate(movement);
    }
}

#[godot_api]
impl Grapes {
    // A method for disapearing grapes when player enters them.
    #[func]
    fn on_body_entered(&mut self, body: Gd<Player>) {
        if body.is_in_group("Player") || body.try_cast::<Player>().is_ok() {
            // try getting the game manager
            if let Some(mut game_manager) = self
                .base()
                .try_get_node_as::<GameManager>("/root/GameManagerGlobal")
            {
                game_manager.bind_mut().add_grapes(1);
            } else {
                godot_warn!("GameManager not an Autoload!");
            }

            if let Some(pickuped) = self.pickup_sound.as_mut() {
                pickuped.play();
            }

            self.base_mut().set_visible(false);
        }
    }
}

// ===================================
// Spawning Grapes
// ===================================

#[godot_api]
impl INode2D for GrapeSpawn {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            spawn_interval: 4.0,
            grape_scene_path: None,
            player_node: None,
            timer: None,
            base,
        }
    }

    fn ready(&mut self) {
        // Find player node
        if let Some(player) = self.base().try_get_node_as::<Player>("../Player") {
            self.player_node = Some(player);
        } else {
            godot_error!("ObstacleSpawn: No Player found at ../Player");
        }

        // start a timer
        let mut timer = Timer::new_alloc();

        // wire in timer timout signal
        timer.connect("timeout", &self.base().callable("on_timer_timeout"));

        // Set random spawn references
        let random_spawn = randf_range(0.5, self.spawn_interval); // spawns anywhere from 0.5 upto max spawn interval
        timer.set_wait_time(random_spawn);
        timer.set_autostart(true);

        // add it to our scene
        self.base_mut().add_child(&timer);
        self.timer = Some(timer);
    }
}

#[godot_api]
impl GrapeSpawn {
    #[func]
    fn on_timer_timeout(&mut self) {
        self.spawn_grapes();
    }

    fn spawn_grapes(&mut self) {
        // Check if player is dead
        if let Some(ref player) = self.player_node {
            if player.bind().is_dead {
                return;
            }
        }

        // Verify that we have a grape scene
        let Some(ref grape_scene) = self.grape_scene_path else {
            godot_error!("No grape scene assigned.");
            return;
        };

        // Finally, we spawn a grape for the hungry crow
        let Some(grape_instance) = grape_scene.instantiate() else {
            return;
        };

        if let Ok(mut grapes_area) = grape_instance.try_cast::<Area2D>() {
            // random position for y spawn
            const MAX_SPAWN_Y: f64 = 275.0;
            let random_y = randf_range(-MAX_SPAWN_Y + 25.0, MAX_SPAWN_Y - 25.0) as f32;

            grapes_area.set_position(Vector2::new(0.0, random_y));
            self.base_mut().add_child(&grapes_area);
        } else {
            godot_error!("Instantiated grape scene root does not inherit from Area2D!");
        }

        // Reset our timer
        if let Some(mut timer) = self.timer.clone() {
            let random_spawn = randf_range(0.5, self.spawn_interval);
            timer.set_wait_time(random_spawn);
        }
    }
}
