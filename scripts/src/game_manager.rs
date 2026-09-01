use godot::classes::file_access::ModeFlags;
use godot::classes::node::ProcessMode;
use godot::classes::ResourceLoader;
use godot::classes::{AudioStreamPlayer2D, DirAccess, INode, Input, Label, Node, TextureRect};
use godot::global::linear_to_db;
use godot::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameManager {
    // Game particulars
    #[export]
    score_increase_rate: f32,
    #[export]
    current_score: i32,
    #[export]
    high_score: i32,

    // State management for menues or game
    current_game_state: GameState,
    pub music_volume: f32,
    pause_menu_instance: Option<Gd<Node>>,

    // UI Interaction for save and game
    #[export]
    current_score_path: NodePath,
    #[export]
    high_score_path: NodePath,
    #[export]
    save_dir: GString,
    float_score: f32,
    is_tracking_score: bool,
    pub grapes: i32,

    // Audio particulars
    game_music: Option<Gd<AudioStreamPlayer2D>>,
    menu_music: Option<Gd<AudioStreamPlayer2D>>,

    base: Base<Node>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SaveData {
    pub high_score: i32,
    pub grapes: i32,
    pub music_volume: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    PlayState,
    TitleState,
    PauseState,
    _PlayerDead,
}

#[godot_api]
impl INode for GameManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            current_game_state: GameState::TitleState,
            music_volume: 0.25,
            score_increase_rate: 2.0,
            current_score: 0,
            high_score: 0,
            current_score_path: NodePath::from(""),
            high_score_path: NodePath::from(""),
            is_tracking_score: false,
            grapes: 0,
            save_dir: "user://saves/".into(),
            float_score: 0.0,
            game_music: None,
            menu_music: None,
            pause_menu_instance: None,
            base,
        }
    }

    fn ready(&mut self) {
        // We need to make sure that the GameManager runs even when the game is paused.
        self.base_mut().set_process_mode(ProcessMode::ALWAYS);

        // If no save loaded, let player know defaults are to be used.
        if !self.load_game() {
            godot_print!("Could not load save file, defaulting to standard values.");
        }

        // Cache the musics
        self.game_music = Some(self.base().get_node_as::<AudioStreamPlayer2D>("GameMusic"));
        self.menu_music = Some(self.base().get_node_as::<AudioStreamPlayer2D>("MenuMusic"));

        self.current_score = 0;
        self.float_score = 0.0;

        self.start_score_tracking();
    }

    fn process(&mut self, delta: f64) {
        let input = Input::singleton();

        // Single source of truth for Escape key handling across the entire game
        if input.is_action_just_pressed("Menu") {
            match self.current_game_state {
                GameState::PlayState => {
                    // Return to the menu
                    self.switch_gamestate(GameState::PauseState);
                }
                GameState::PauseState => {
                    // Back out of the menu into gameplay
                    self.switch_gamestate(GameState::PlayState);
                }
                GameState::TitleState => {
                    // This is just blank
                }
                _ => {
                    todo!()
                    // Need to add in a game over overlay, subset of the menu scene?
                }
            }
            return;
        }

        // Accumulate score only while playing
        if self.current_game_state == GameState::PlayState && self.is_tracking_score {
            self.float_score += (delta as f32) * self.score_increase_rate;

            let added_score = self.float_score.floor() as i32;
            self.current_score += added_score;
            self.float_score -= added_score as f32;
            self.update_score_ui();

            if self.current_score > self.high_score {
                self.high_score = self.current_score;
            }
        }
    }
}

#[godot_api]
impl GameManager {
    /// Core state transition handler
    pub fn switch_gamestate(&mut self, new_state: GameState) {
        let old_state = self.current_game_state;
        self.current_game_state = new_state;

        match new_state {
            GameState::PlayState => {
                if old_state == GameState::PauseState {
                    self.resume_game();
                } else {
                    // First, Reset current session score
                    self.current_score = 0;
                    self.float_score = 0.0;

                    // Then, start tracking score and change music
                    self.start_score_tracking();
                    self.play_game_music();

                    // Load up game scene
                    if let Some(mut tree) = self.base().get_tree().into() {
                        tree.change_scene_to_file("res://scenes/game.tscn");
                    }
                }
            }
            GameState::PauseState => {
                self.pause_game();
            }
            GameState::TitleState => {
                // First, Stop tracking score & auto-save high scores
                self.stop_score_tracking();

                // Then, switch to menu music
                self.play_menu_music();

                // Load main menu scene
                if let Some(mut tree) = self.base().get_tree().into() {
                    tree.change_scene_to_file("res://scenes/main_menu.tscn");
                }
            }
            _ => {}
        }
    }

    fn pause_game(&mut self) {
        self.is_tracking_score = false;
        self.play_menu_music();

        // Freeze game engine node processing
        if let Some(mut tree) = self.base().get_tree().into() {
            tree.set_pause(true);
        }

        // Load and instantiate the menu scene as an overlay
        let mut loader = ResourceLoader::singleton();
        if let Some(res) = loader.load("res://scenes/main_menu.tscn") {
            if let Ok(packed_scene) = res.try_cast::<PackedScene>() {
                if let Some(overlay) = packed_scene.instantiate() {
                    if let Some(mut background) =
                        overlay.try_get_node_as::<TextureRect>("Background")
                    {
                        background.set_visible(false);
                    }

                    // Cache the reference so resume_game can queue_free()
                    self.pause_menu_instance = Some(overlay.clone());

                    // Attaching overlay to scene
                    if let Some(tree) = self.base().get_tree().into() {
                        if let Some(mut current_scene) = tree.get_current_scene() {
                            current_scene.call_deferred("add_child", &[overlay.to_variant()]);
                            self.pause_menu_instance = Some(overlay);
                        }
                    }
                }
            }
        }
    }

    fn resume_game(&mut self) {
        // Unfreeze engine node processing
        if let Some(mut tree) = self.base().get_tree().into() {
            tree.set_pause(false);
        }

        // Remove menu overlay
        if let Some(mut overlay) = self.pause_menu_instance.take() {
            overlay.queue_free();
        }

        self.is_tracking_score = true;
        self.play_game_music();
    }

    #[func]
    pub fn change_to_play_state(&mut self) {
        self.switch_gamestate(GameState::PlayState);
    }

    #[func]
    pub fn change_to_menu_state(&mut self) {
        self.switch_gamestate(GameState::TitleState);
    }

    #[func]
    pub fn start_score_tracking(&mut self) {
        self.is_tracking_score = true;
    }

    #[func]
    pub fn stop_score_tracking(&mut self) {
        if self.current_score >= self.high_score {
            self.high_score = self.current_score;
        }
        self.is_tracking_score = false;
        self.save_game();
    }

    #[func]
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
        self.apply_volume();
        self.save_game();
    }

    #[func]
    pub fn apply_volume(&mut self) {
        // Convert the linear volume (0.0 to 1.0) to a dB value.
        let db = linear_to_db(self.music_volume as f64) as f32;

        if let Some(mut game_music) = self.game_music.clone() {
            game_music.set_volume_db(db);
        }
        if let Some(mut menu_music) = self.menu_music.clone() {
            menu_music.set_volume_db(db);
        }
    }

    #[func]
    pub fn play_menu_music(&mut self) {
        // Stop game music if it's playing
        if let Some(game_music) = self.game_music.as_mut() {
            game_music.stop();
        }

        // Play menu music
        if let Some(menu_music) = self.menu_music.as_mut() {
            if !menu_music.is_playing() {
                menu_music.play();
            }
        }
        self.apply_volume();
    }

    #[func]
    pub fn play_game_music(&mut self) {
        if let Some(menu_music) = self.menu_music.as_mut() {
            menu_music.stop();
        }

        if let Some(game_music) = self.game_music.as_mut() {
            if !game_music.is_playing() {
                game_music.play();
            }
        }
        self.apply_volume();
    }

    #[func]
    pub fn save_game(&self) -> bool {
        let full_path = format!("{}save_game.json", self.save_dir).to_gstring();

        if !DirAccess::dir_exists_absolute(&self.save_dir.clone()) {
            DirAccess::make_dir_recursive_absolute(&self.save_dir.clone());
        }

        let the_stuff = SaveData {
            high_score: self.high_score,
            grapes: self.grapes,
            music_volume: self.music_volume,
        };

        match GFile::open(&full_path, ModeFlags::WRITE) {
            Ok(mut my_file) => {
                if let Err(e) = serde_json::to_writer_pretty(&mut my_file, &the_stuff) {
                    godot_print!("Serialization error: {:?}", e);
                    return false;
                }
                godot_print!("Game saved to {}", full_path);
                true
            }
            Err(e) => {
                godot_print!("Failed to open save file: {:?}", e);
                false
            }
        }
    }

    #[func]
    pub fn load_game(&mut self) -> bool {
        let full_path = format!("{}save_game.json", self.save_dir).to_gstring();

        match GFile::open(&full_path, ModeFlags::READ) {
            Ok(the_file) => match serde_json::from_reader::<_, SaveData>(the_file) {
                Ok(loaded_data) => {
                    self.high_score = loaded_data.high_score;
                    self.grapes = loaded_data.grapes;
                    self.music_volume = loaded_data.music_volume;
                    godot_print!(
                        "Game loaded from {}: High Score: {}, Grapes: {}",
                        full_path,
                        self.high_score,
                        self.grapes,
                    );
                    true
                }
                Err(e) => {
                    let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, e);
                    godot_error!("Save file corrupted: {}", io_err);
                    false
                }
            },
            Err(e) => {
                godot_print!("No save file found or accessable at {}: {:?}", full_path, e);
                false
            }
        }
    }

    fn update_score_ui(&mut self) {
        let Some(tree) = self.base().get_tree().into() else {
            return;
        };
        let Some(current_scene) = tree.get_current_scene() else {
            return;
        };

        // Now we query for the score label using relative paths to the game.tscn root
        // Adjust the "HUD/Score" paths to match game.tscn.
        if let Some(mut score_label) = current_scene.try_get_node_as::<Label>("HUD/Score") {
            score_label.set_text(&self.current_score.to_string());
        } else {
            godot_warn!("Could not find 'Score' inside current scene.");
        }

        //Then adjust high score
        if let Some(mut high_score_label) = current_scene.try_get_node_as::<Label>("HUD/HighScore")
        {
            high_score_label.set_text(&self.high_score.to_string());
        }

        if let Some(mut grapes_label) = current_scene.try_get_node_as::<Label>("HUD/Grapes") {
            grapes_label.set_text(&self.grapes.to_string());
        }
    }

    // adjust grape counts
    pub fn add_grapes(&mut self, amount: i32) {
        self.grapes += amount;
        self.update_score_ui();
    }
}
