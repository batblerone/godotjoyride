use godot::classes::{Area2D, IArea2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct DespawnArea {
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for DespawnArea {
    fn init(base: Base<Area2D>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl DespawnArea {
    #[func]
    // This isn't despawning things, probably cause I am calling area2d, 
    // and not making a universl despawn for my custom nodes.
    fn destroy_entity(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("Obstacles") || area.is_in_group("Tokens") {
            let entity_name = area.get_name();
            godot_print!("Despawned {}", entity_name);
            let mut entity = area;
            entity.queue_free();
        }
    }
}
