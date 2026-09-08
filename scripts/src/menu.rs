use crate::game_manager::*;
use godot::classes::window::Mode;
use godot::classes::{AudioServer, CanvasLayer, HSlider, ICanvasLayer, VBoxContainer};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=CanvasLayer)]
pub struct MenuLayer {
    current_state: MenuState,
    base: Base<CanvasLayer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    Main,
    Settings,
    Credits,
    GameOver,
}

#[godot_api]
impl ICanvasLayer for MenuLayer {
    fn init(base: Base<CanvasLayer>) -> Self {
        Self {
            current_state: MenuState::Main,
            base,
        }
    }

    fn ready(&mut self) {
        // Initialize menu states
        self.switch_to_menu(MenuState::Main);
        if let Some(mut game_manager_node) = self
            .base()
            .get_node_as::<GameManager>("/root/GameManagerGlobal")
            .into()
        {
            let mut game_manager = game_manager_node.bind_mut();

            // First play some music
            game_manager.play_menu_music();

            // Next fetch a saved volume
            let saved_volume = game_manager.music_volume;

            // Apply the initial volume to the Audio Server
            self.apply_volume_to_audio_server(saved_volume);

            // Laslty, set the position of the slider in the UI
            if let Some(mut slider) = self
                .base()
                .try_get_node_as::<HSlider>("ButtonContainer/SettingsMenu/VolumeSlider")
            {
                slider.set_value(saved_volume as f64);
            } else {
                godot_warn!("HSlider not found at path.");
            }
        } else {
            godot_error!("GameManagerGlobal Autoload not found at /root/GameManagerGlobal");
        }
    }
}

#[godot_api]
impl MenuLayer {
    pub fn switch_to_menu(&mut self, new_state: MenuState) {
        // Switch the state for menu lookups.
        self.current_state = new_state;

        // Get the nodes for the menus
        let main_menu = self
            .base()
            .try_get_node_as::<VBoxContainer>("ButtonContainer/MainMenu");
        let settings_menu = self
            .base()
            .try_get_node_as::<VBoxContainer>("ButtonContainer/SettingsMenu");
        let credits_menu = self
            .base()
            .try_get_node_as::<VBoxContainer>("ButtonContainer/CreditsMenu");
        let game_over = self
            .base()
            .try_get_node_as::<VBoxContainer>("ButtonContainer/GameOver");

        // Adjust the visibility
        if let Some(mut main) = main_menu {
            main.set_visible(self.current_state == MenuState::Main);
        }

        if let Some(mut settings) = settings_menu {
            settings.set_visible(self.current_state == MenuState::Settings);
        }

        if let Some(mut credits) = credits_menu {
            credits.set_visible(self.current_state == MenuState::Credits);
        }

        if let Some(mut end_game) = game_over {
            end_game.set_visible(self.current_state == MenuState::GameOver);
        }
    }

    // Expose the methods to godot to wire up to buttons
    #[func]
    fn show_main(&mut self) {
        self.switch_to_menu(MenuState::Main);
    }
    #[func]
    fn show_settings(&mut self) {
        self.switch_to_menu(MenuState::Settings);
    }
    #[func]
    fn show_credits(&mut self) {
        self.switch_to_menu(MenuState::Credits);
    }
    #[func]
    fn show_game_over(&mut self) {
        self.switch_to_menu(MenuState::GameOver);
    }

    #[func]
    fn on_play_pressed(&mut self) {
        if let Some(mut game_manager_node) = self
            .base()
            .try_get_node_as::<GameManager>("/root/GameManagerGlobal")
        {
            let mut game_manager = game_manager_node.bind_mut();
            game_manager.change_to_play_state();
        } else {
            godot_error!("GameManagerGlobal not found in Autoload.");
        }
    }

    #[func]
    fn on_fullscreen_toggle(&mut self, toggled_on: bool) {
        if let Some(mut window) = self.base().get_window() {
            if toggled_on {
                window.set_mode(Mode::FULLSCREEN);
            } else {
                window.set_mode(Mode::WINDOWED);
            }
        }
    }

    #[func]
    fn on_slider_changed(&mut self, value: f32) {
        let mut audio_server = AudioServer::singleton();
        let bus_index = audio_server.get_bus_index("Master");

        audio_server.set_bus_volume_linear(bus_index, value);
        if value == 0.0 {
            audio_server.set_bus_mute(bus_index, true);
        } else {
            audio_server.set_bus_mute(bus_index, false);
        }
    }

    fn apply_volume_to_audio_server(&self, value: f32) {
        let mut audio_server = AudioServer::singleton();
        let bus_index = audio_server.get_bus_index("Master");

        audio_server.set_bus_volume_linear(bus_index, value);
        audio_server.set_bus_mute(bus_index, value <= 0.0);
    }

    #[func]
    fn on_exit_pressed(&mut self) {
        self.base().get_tree().quit();
    }
}
