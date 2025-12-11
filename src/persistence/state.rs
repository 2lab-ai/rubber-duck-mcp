use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::world::*;
use crate::entity::*;
use rand::Rng;

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
    #[serde(default)]
    pub trees: Vec<Tree>,
    #[serde(default)]
    pub custom_names: HashMap<Item, String>,
    // Runtime state (not critical to save but nice to have)
    #[serde(default)]
    pub pending_messages: Vec<String>,
}

impl GameState {
    /// Create a new game state with initial values
    pub fn new(map: &WorldMap) -> Self {
        let mut rng = rand::thread_rng();
        let mut state = Self {
            version: "1.0".to_string(),
            time: WorldTime::new(),
            weather: RegionalWeather::new(),
            player: Player::new(),
            cabin: Cabin::new(),
            wood_shed: WoodShed::new(),
            wildlife: spawn_wildlife(),
            trees: Tree::default_trees(),
            custom_names: HashMap::new(),
            pending_messages: Vec::new(),
        };
        state.seed_tree_population(map, &mut rng, 10);
        state
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
    pub fn load_or_new(path: &Path, map: &WorldMap) -> Self {
        if path.exists() {
            match Self::load(path) {
                Ok(mut state) => {
                    tracing::info!("Loaded existing game state from {:?}", path);
                    // Auto-spawn wildlife if empty
                    if state.wildlife.is_empty() {
                        tracing::info!("Wildlife was empty, spawning new wildlife");
                        state.wildlife = spawn_wildlife();
                    }
                    if state.trees.is_empty() {
                        tracing::info!("Tree list missing or empty, seeding defaults");
                        state.trees = Tree::default_trees();
                    }
                    for t in &mut state.trees {
                        t.apply_kind_defaults();
                    }
                    let mut rng = rand::thread_rng();
                    state.seed_tree_population(map, &mut rng, 10);
                    // Ensure key cabin items exist for new features
                    if !state.cabin.items.contains(&Item::Kettle) {
                        tracing::info!("Adding missing kettle to cabin inventory");
                        state.cabin.items.push(Item::Kettle);
                    }
                    if !state.cabin.items.contains(&Item::TeaCup) {
                        tracing::info!("Adding missing tea cup to cabin inventory");
                        state.cabin.items.push(Item::TeaCup);
                    }
                    if !state.cabin.items.contains(&Item::WildHerbs) {
                        tracing::info!("Adding starter herbs to cabin inventory");
                        state.cabin.items.push(Item::WildHerbs);
                    }
                    if state.custom_names.is_empty() {
                        state.custom_names = HashMap::new();
                    }
                    if !state.cabin.table_items.contains(&Item::RubberDuck)
                        && !state.cabin.items.contains(&Item::RubberDuck)
                        && !state.player.inventory.has(&Item::RubberDuck, 1)
                    {
                        tracing::info!("Placing rubber duck on the cabin table");
                        state.cabin.table_items.push(Item::RubberDuck);
                    }
                    state
                }
                Err(e) => {
                    tracing::warn!("Failed to load state: {}, creating new", e);
                    Self::new(map)
                }
            }
        } else {
            tracing::info!("No save file found, creating new game state");
            Self::new(map)
        }
    }

    /// Advance the simulation by one tick
    pub fn tick_with_map(&mut self, map: &WorldMap) {
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

        // Update fireplace and collect any warnings
        if let Some(fire_msg) = self.cabin.fireplace.update() {
            self.pending_messages.push(fire_msg);
        }

        let mut rng = rand::thread_rng();
        self.update_trees(map, &mut rng);

        // Hunger / thirst decay
        self.player.modify_fullness(-0.5);
        self.player.modify_hydration(-0.5);
        if self.player.fullness < 20.0 {
            self.player.modify_energy(-1.0);
            self.player.modify_mood(-1.0);
            if self.player.fullness < 10.0 {
                self.pending_messages.push("Your stomach growls painfully. You need to eat soon.".to_string());
            }
        }
        if self.player.hydration < 20.0 {
            self.player.modify_energy(-1.0);
            if self.player.hydration < 10.0 {
                self.player.modify_health(-0.5);
                self.pending_messages.push("Your mouth is dry and head swims. Drink water soon.".to_string());
            }
        }

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

    fn living_tree_count(&self) -> usize {
        self.trees.iter().filter(|t| !t.felled).count()
    }

    fn find_free_tree_spot(
        &self,
        map: &WorldMap,
        rng: &mut impl Rng,
        attempts: usize,
    ) -> Option<Position> {
        for _ in 0..attempts {
            let row = rng.gen_range(0..MAP_HEIGHT as i32);
            let col = rng.gen_range(0..MAP_WIDTH as i32);
            let pos = Position::new(row, col);
            if !pos.is_valid() {
                continue;
            }
            if self.trees.iter().any(|t| t.position == pos) {
                continue;
            }
            if let Some(tile) = map.get_tile(row as usize, col as usize) {
                if matches!(tile.tile_type, TileType::Forest(_)) && tile.walkable {
                    return Some(pos);
                }
            }
        }
        None
    }

    fn random_tree_kind(&self, rng: &mut impl Rng) -> TreeType {
        match rng.gen_range(0..3) {
            0 => TreeType::Pine,
            1 => TreeType::Birch,
            _ => TreeType::Apple,
        }
    }

    fn spawn_tree(&mut self, map: &WorldMap, rng: &mut impl Rng) -> bool {
        let Some(pos) = self.find_free_tree_spot(map, rng, 50) else {
            return false;
        };
        let kind = self.random_tree_kind(rng);
        let tree = Tree::with_random_fruit(pos, kind, rng);
        self.trees.push(tree);
        true
    }

    fn seed_tree_population(&mut self, map: &WorldMap, rng: &mut impl Rng, target: usize) {
        while self.living_tree_count() < target {
            if !self.spawn_tree(map, rng) {
                break;
            }
        }
    }

    fn update_trees(&mut self, map: &WorldMap, rng: &mut impl Rng) {
        for tree in &mut self.trees {
            tree.tick_growth(rng);
        }
        if self.living_tree_count() <= 5 {
            let _ = self.spawn_tree(map, rng);
        }
    }

    pub fn set_custom_name(&mut self, item: Item, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            self.custom_names.remove(&item);
            return;
        }
        let capped = trimmed.chars().take(32).collect::<String>();
        self.custom_names.insert(item, capped);
    }

    pub fn custom_name(&self, item: &Item) -> Option<&str> {
        self.custom_names.get(item).map(|s| s.as_str())
    }

    pub fn display_name(&self, item: &Item) -> String {
        self.custom_name(item).map(|s| s.to_string()).unwrap_or_else(|| item.name().to_string())
    }

    pub fn player_can_access_item(&self, item: &Item) -> bool {
        if self.player.inventory.has(item, 1) {
            return true;
        }
        if matches!(self.player.room, Some(Room::CabinMain)) {
            if self.cabin.items.contains(item) || self.cabin.table_items.contains(item) {
                return true;
            }
        }
        false
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(&WorldMap::new())
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
        let map = WorldMap::new();
        let state = GameState::load_or_new(&state_path, &map);
        Self { map, state, state_path }
    }

    pub fn save(&self) -> Result<()> {
        self.state.save(&self.state_path)
    }

    pub fn tick(&mut self) {
        self.state.tick_with_map(&self.map);
    }
}
