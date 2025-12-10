use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::world::*;
use crate::entity::*;

/// The complete game state that gets saved/loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub version: String,
    pub time: WorldTime,
    pub weather: RegionalWeather,
    pub player: Player,
    pub cabin: Cabin,
    pub wood_shed: WoodShed,
    pub wildlife: Vec<Wildlife>,
    // Map is regenerated, not saved
}

impl GameState {
    /// Create a new game state with initial values
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            time: WorldTime::new(),
            weather: RegionalWeather::new(),
            player: Player::new(),
            cabin: Cabin::new(),
            wood_shed: WoodShed::new(),
            wildlife: spawn_wildlife(),
        }
    }

    /// Save state to a JSON file
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load state from a JSON file
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let state: GameState = serde_json::from_str(&json)?;
        Ok(state)
    }

    /// Load state or create new if file doesn't exist
    pub fn load_or_new(path: &Path) -> Self {
        if path.exists() {
            match Self::load(path) {
                Ok(state) => {
                    tracing::info!("Loaded existing game state from {:?}", path);
                    state
                }
                Err(e) => {
                    tracing::warn!("Failed to load state: {}, creating new", e);
                    Self::new()
                }
            }
        } else {
            tracing::info!("No save file found, creating new game state");
            Self::new()
        }
    }

    /// Advance the simulation by one tick
    pub fn tick(&mut self) {
        // Advance time
        self.time.advance_tick();

        // Update weather occasionally
        if self.time.tick % 10 == 0 {
            self.weather.update();
        }

        // Update wildlife
        let tod = self.time.time_of_day();
        for w in &mut self.wildlife {
            w.update(tod);
        }

        // Update fireplace
        self.cabin.fireplace.update();

        // Update player warmth based on environment
        self.update_player_comfort();
    }

    fn update_player_comfort(&mut self) {
        let indoor = self.player.is_indoor();
        let fire_heat = if matches!(self.player.room, Some(Room::CabinMain)) {
            self.cabin.fireplace.heat_output()
        } else {
            0.0
        };

        // Get position for temperature calculation
        let (row, col) = self.player.position.as_usize().unwrap_or((5, 5));

        // This is simplified - would need map reference for full calculation
        let base_temp = match self.player.room {
            Some(_) if fire_heat > 0.0 => 15.0 + fire_heat,
            Some(_) => 12.0, // Indoor base temp
            None => {
                // Outdoor - use weather
                let weather_temp = self.weather.get_for_position(row as i32, col as i32)
                    .temperature_modifier();
                15.0 + weather_temp
            }
        };

        // Adjust player warmth toward environmental temperature
        let comfort_target = (base_temp + 20.0).clamp(0.0, 100.0);
        let current = self.player.warmth;
        let delta = (comfort_target - current) * 0.1; // Gradual change
        self.player.modify_warmth(delta);

        // Mood effects from comfort
        if self.player.warmth > 40.0 && self.player.warmth < 60.0 {
            self.player.modify_mood(0.5); // Comfortable = happier
        } else if self.player.warmth < 30.0 || self.player.warmth > 70.0 {
            self.player.modify_mood(-0.5); // Uncomfortable = less happy
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

/// Full world context including map (which isn't saved)
pub struct World {
    pub map: WorldMap,
    pub state: GameState,
    pub state_path: std::path::PathBuf,
}

impl World {
    pub fn new(state_path: std::path::PathBuf) -> Self {
        let state = GameState::load_or_new(&state_path);
        Self {
            map: WorldMap::new(),
            state,
            state_path,
        }
    }

    pub fn save(&self) -> Result<()> {
        self.state.save(&self.state_path)
    }

    pub fn tick(&mut self) {
        self.state.tick();
    }
}
