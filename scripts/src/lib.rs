use godot::prelude::*;

mod background;
mod decimator;
mod game_manager;
mod grape;
mod menu;
mod obstacles;
mod player;

struct GodotJoyrideExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotJoyrideExtension {}
