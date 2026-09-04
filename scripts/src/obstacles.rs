use crate::player::*;
use godot::classes::{
    AnimatedSprite2D, Area2D, CharacterBody2D, IArea2D, ICharacterBody2D, INode2D, Node2D, Timer,
};
use godot::global::randf_range;
use godot::prelude::*;

// A node that defines particulars for the obstacle spawner
#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct ObstacleSpawn {
    #[export]
    spawn_interval: f64,
    #[export]
    obstacle_scene_path: Array<Option<Gd<PackedScene>>>,
    #[export]
    obstacle_rarity: Array<i32>,
    player_node: Option<Gd<Player>>,
    timer: Option<Gd<Timer>>,
    base: Base<Node2D>,
}

// This node is for the owls
#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Owls {
    #[export]
    owl_speed: f32,
    animation_player: Option<Gd<AnimatedSprite2D>>,
    base: Base<CharacterBody2D>,
}

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct TreeBranch {
    tree_speed: f32,
    is_rotating: bool,
    rotation_speed: f32,
    base: Base<Area2D>,
}

// =======================================
// Spawning Obstacles
// =======================================
#[godot_api]
impl INode2D for ObstacleSpawn {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            spawn_interval: 8.0,
            obstacle_scene_path: Array::new(),
            obstacle_rarity: iarray![40, 35, 25],
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
impl ObstacleSpawn {
    #[func]
    fn on_timer_timeout(&mut self) {
        self.spawn_obstacle();
    }

    fn spawn_obstacle(&mut self) {
        // Check if player is dead
        if let Some(ref player) = self.player_node {
            if player.bind().is_dead {
                return;
            }
        }

        // Verify the obstacle array isn't empty
        if self.obstacle_scene_path.is_empty() {
            godot_error!("Obstacle scene array is empty. Assign scenes in inspector to continue");
            return;
        }

        // A weighted choice for the obstacles
        let spawned_type = self.randomized_obstacle();
        // Map weighted outcomes to the scene indices
        let target_scene_index = match spawned_type {
            0 => 0,
            1 | 2 => 1,
            _ => return,
        };

        // Retrieve PackedScene, tells you there isn't a valid scene at index if error.
        let Some(Some(ref packed_scene)) = self.obstacle_scene_path.get(target_scene_index) else {
            godot_error!("No valid scene at index {}", target_scene_index);
            return;
        };

        // Instantiate the packed scene we retrieved, error if it just doesn't.
        let Some(instance) = packed_scene.instantiate() else {
            return;
        };

        // Then cast the scene up to the Node2D parent so we can work with it.
        let Ok(mut root) = instance.try_cast::<Node2D>() else {
            godot_error!("Obstacle scene root does not inherit Node2D.");
            return;
        };

        const MAX_SPAWN_Y: f64 = 275.0;
        let mut spawn_position = Vector2::ZERO;

        // MAKE SURE YOUR SCENES IN GODOT MATCH THIS - 0 IS OWL, 1 IS TREES
        match spawned_type {
            0 => {
                // Child positions are local to spawner, we need to convert
                if let Some(ref player) = self.player_node {
                    spawn_position.y =
                        player.get_global_position().y - self.base().get_global_position().y;
                }
            }
            1 | 2 => {
                spawn_position.y = randf_range(-MAX_SPAWN_Y + 50.0, MAX_SPAWN_Y - 50.0) as f32;

                // Now rotate them branches, rotate node itself, not scene root
                match Self::find_branch(&root) {
                    Some(mut branch) => {
                        branch.set_rotation(randf_range(0.0, std::f64::consts::TAU) as f32);
                        if spawned_type == 2 {
                            branch.bind_mut().is_rotating = true;
                        }
                    }
                    None => {
                        godot_error!("Tree scene contains no TreeBranch node.");
                    }
                }
            }
            _ => {}
        }

        root.set_position(spawn_position);
        self.base_mut().add_child(&root);

        if let Some(timer) = self.timer.as_mut() {
            timer.set_wait_time(randf_range(0.5, self.spawn_interval));
            timer.start(); // Changing wait time alone doesnt affect cycle, have to start it agian.
        }
    }

    // Accepts TreeBranch as the root or as a direct child
    fn find_branch(node: &Gd<Node2D>) -> Option<Gd<TreeBranch>> {
        if let Ok(branch) = node.clone().try_cast::<TreeBranch>() {
            return Some(branch);
        }
        node.get_children()
            .iter_shared()
            .find_map(|child| child.try_cast::<TreeBranch>().ok())
    }

    fn randomized_obstacle(&mut self) -> i32 {
        if self.obstacle_rarity.is_empty() {
            return -1;
        }

        // Sum our weights
        let total_weight: i32 = self.obstacle_rarity.iter_shared().sum();
        if total_weight <= 0 {
            return -1;
        }

        // generate a random number up to total weight
        let random_weight = godot::global::randi() % total_weight as i64;
        let mut cumulative_weight: i64 = 0;

        // iterate through elements safely using shared loop state
        for (index, weight) in self.obstacle_rarity.iter_shared().enumerate() {
            cumulative_weight += weight as i64;
            if random_weight < cumulative_weight {
                return index as i32;
            }
        }

        -1
    }
}

// =======================================
// Handling Spawned Owls
// =======================================
#[godot_api]
impl ICharacterBody2D for Owls {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            owl_speed: -6.0,
            animation_player: None,
            base,
        }
    }

    fn ready(&mut self) {
        if let Some(mut animated) = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite2D")
        {
            animated.play();
            self.animation_player = Some(animated);
        } else {
            godot_print!("Warning: AnimationPlayer node not found at root.");
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // Debug for check loop fire
        // godot_print!("Owls fly_speed: {}", self.owl_speed);

        // Slllliiiiiide to the left (actually move, not slide)
        let movement = Vector2::new(self.owl_speed, 0.0) * delta as f32 * 80.0;
        let mut owls = self.base_mut();
        owls.translate(movement);
    }
}

// =======================================
// Handling spawned Trees
// =======================================
#[godot_api]
impl IArea2D for TreeBranch {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            tree_speed: -6.0,
            is_rotating: false,
            rotation_speed: 10.0,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // Debug for check loop fire
        // godot_print!("Trees fly_speed: {}", self.tree_speed);

        // Slllliiiiiide to the left (actually move, not slide)
        let movement = Vector2::new(self.tree_speed, 0.0) * delta as f32 * 80.0;
        let is_rotating = self.is_rotating;
        let rotation_speed = self.rotation_speed;

        let mut trees = self.base_mut();
        let current_pos = trees.get_position();
        trees.set_position(current_pos + movement);
        if is_rotating {
            trees.rotate(rotation_speed * delta as f32);
        }
    }
}
