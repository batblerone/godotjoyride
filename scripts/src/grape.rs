use crate::game_manager::*;
use crate::player::*;
use godot::classes::{AnimatedSprite2D, Area2D, IArea2D, Node2D, Timer};
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
impl INode2D for GrapeSpawn {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            spawn_interval: 2.5,
            grape_scene_path: None,
            player_node: None,
            timer: None,
            base,
        }
    }

    fn ready(&mut self) {
        // Find player node
        if let Some(player_node) = self.base().get_node_or_null("../Player") {
            let player_gd: Gd<Player> = player_node.cast::<Player>();

            let player_struct = player_gd.bind();

            if player_struct.is_dead {
                return;
            }
        }

        // Start a timer
        let mut timer = Timer::new_alloc();

        // Wire up a timer timeout signal
        timer.connect("timeout", &self.base().callable("on_timer_timeout"));

        timer.set_wait_time(self.spawn_interval);
        timer.set_autostart(true);

        // Now add this all into our scene
        {
            let mut base = self.base_mut();
            base.add_child(&timer);
        }

        self.timer = Some(timer);
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
            self.base_mut().queue_free();
        }
    }
}

#[godot_api]
impl GrapeSpawn {
    #[func]
    fn on_timer_timeout(&mut self) {
        self.spawn_grapes();
    }

    fn spawn_grapes(&mut self) {
        // Check the players dead status
        if let Some(ref player) = self.player_node {
            if player.get("is_dead").to::<bool>() {
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
        let mut grape_area = grape_instance.cast::<Grapes>();

        // DEBUG: Spawn grapes check.
        // let grape_name = grape_area.get_name();
        // godot_print!("Spawned {}", grape_name);

        self.base_mut().add_child(&grape_area);

        // random position for y spawn
        const MAX_SPAWN_Y: f64 = 275.0;
        let random_y = randf_range(-MAX_SPAWN_Y + 25.0, MAX_SPAWN_Y - 25.0) as f32;

        grape_area.set_position(Vector2::new(0.0, random_y));
    }
}
