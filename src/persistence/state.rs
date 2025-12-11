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
    pub wildlife: Vec<Wildlife>,
    #[serde(default)]
    pub objects: ObjectRegistry,
    #[serde(default)]
    pub custom_names: HashMap<Item, String>,
    // Runtime state (not critical to save but nice to have)
    #[serde(default)]
    pub pending_messages: Vec<String>,
    #[serde(default, rename = "cabin")]
    #[serde(skip_serializing)]
    legacy_cabin: Option<Cabin>,
    #[serde(default, rename = "wood_shed")]
    #[serde(skip_serializing)]
    legacy_wood_shed: Option<WoodShed>,
    #[serde(default, rename = "trees")]
    #[serde(skip_serializing)]
    legacy_trees: Option<Vec<Tree>>,
}

impl GameState {
    pub fn cabin_state(&self) -> Option<&Cabin> {
        self.objects
            .find("cabin")
            .and_then(|p| p.object.as_cabin())
    }

    pub fn cabin_state_mut(&mut self) -> Option<&mut Cabin> {
        self.objects
            .find_mut("cabin")
            .and_then(|p| p.object.as_cabin_mut())
    }

    pub fn wood_shed_state(&self) -> Option<&WoodShed> {
        self.objects
            .find("wood_shed")
            .and_then(|p| p.object.as_wood_shed())
    }

    pub fn wood_shed_state_mut(&mut self) -> Option<&mut WoodShed> {
        self.objects
            .find_mut("wood_shed")
            .and_then(|p| p.object.as_wood_shed_mut())
    }

    pub fn table_surface(&self) -> Option<&ObjectSurface> {
        self.objects
            .find("cabin_table")
            .and_then(|p| p.object.surface.as_ref())
    }

    pub fn table_surface_mut(&mut self) -> Option<&mut ObjectSurface> {
        self.objects
            .find_mut("cabin_table")
            .and_then(|p| p.object.surface.as_mut())
    }

    fn ensure_core_cabin_items(cabin: &mut Cabin) {
        if !cabin.items.contains(&Item::Kettle) {
            cabin.items.push(Item::Kettle);
        }
        if !cabin.items.contains(&Item::TeaCup) {
            cabin.items.push(Item::TeaCup);
        }
        if !cabin.items.contains(&Item::WildHerbs) {
            cabin.items.push(Item::WildHerbs);
        }
    }

    fn ensure_table_object(&mut self, mut table_items: Vec<Item>) {
        if let Some(table) = self.objects.find_mut("cabin_table") {
            if let Some(surface) = table.object.surface.as_mut() {
                surface.items.extend(table_items.drain(..));
                surface.supports_mounts = true;
                if surface.capacity.is_none() {
                    surface.capacity = Some(8);
                }
            } else {
                table.object.surface = Some(ObjectSurface {
                    items: table_items,
                    capacity: Some(8),
                    supports_mounts: true,
                });
            }
            return;
        }

        let mut table_obj = WorldObject::new(ObjectKind::Table);
        if let Some(surface) = table_obj.surface.as_mut() {
            surface.items.extend(table_items.drain(..));
            surface.capacity = Some(8);
            surface.supports_mounts = true;
        }
        self.objects
            .add("cabin_table", Position::new(6, 5), table_obj);
    }

    fn ensure_duck_present(&mut self) {
        let duck = Item::RubberDuck;
        let duck_on_table = self
            .table_surface()
            .map(|s| s.items.contains(&duck))
            .unwrap_or(false);
        let duck_in_cabin = self
            .cabin_state()
            .map(|c| c.items.contains(&duck) || c.table_items.contains(&duck))
            .unwrap_or(false);
        let duck_with_player = self.player.inventory.has(&duck, 1);

        if duck_on_table || duck_in_cabin || duck_with_player {
            return;
        }

        if let Some(surface) = self.table_surface_mut() {
            surface.items.push(duck);
            return;
        }

        if let Some(cabin) = self.cabin_state_mut() {
            cabin.items.push(duck);
        }
    }

    fn bootstrap_structures(&mut self) {
        let mut cabin_state = self.legacy_cabin.take().unwrap_or_else(Cabin::new);
        Self::ensure_core_cabin_items(&mut cabin_state);
        let mut table_items = std::mem::take(&mut cabin_state.table_items);

        if self.objects.find("cabin").is_none() {
            self.objects.add(
                "cabin",
                Position::new(6, 5),
                WorldObject::new(ObjectKind::Cabin(cabin_state)),
            );
        } else if let Some(po) = self.objects.find_mut("cabin") {
            if let Some(cabin) = po.object.as_cabin_mut() {
                Self::ensure_core_cabin_items(cabin);
                if table_items.is_empty() && !cabin.table_items.is_empty() {
                    table_items.extend(cabin.table_items.iter().copied());
                }
            }
        }

        let wood_shed_state = self
            .legacy_wood_shed
            .take()
            .unwrap_or_else(WoodShed::new);
        if self.objects.find("wood_shed").is_none() {
            self.objects.add(
                "wood_shed",
                Position::new(5, 4),
                WorldObject::new(ObjectKind::WoodShed(wood_shed_state)),
            );
        } else if let Some(po) = self.objects.find_mut("wood_shed") {
            if po.object.as_wood_shed().is_none() {
                po.object.kind = ObjectKind::WoodShed(wood_shed_state);
            }
        }

        self.ensure_table_object(table_items);
        self.ensure_duck_present();
    }

    fn ensure_tree_objects_from_legacy(&mut self) {
        if let Some(legacy) = self.legacy_trees.take() {
            for mut tree in legacy {
                tree.apply_kind_defaults();
                let pos = tree.position;
                let id = format!("tree-{}-{}-legacy", pos.row, pos.col);
                self.objects
                    .add(id, pos, WorldObject::new(ObjectKind::Tree(tree)));
            }
        }
    }

    pub fn take_table_item(&mut self, item: &Item) -> bool {
        if let Some(surface) = self.table_surface_mut() {
            return surface.take_item(item);
        }
        if let Some(cabin) = self.cabin_state_mut() {
            return cabin.take_table_item(item);
        }
        false
    }

    pub fn add_table_item(&mut self, item: Item) {
        if let Some(surface) = self.table_surface_mut() {
            surface.add_item(item);
            return;
        }
        if let Some(cabin) = self.cabin_state_mut() {
            cabin.add_table_item(item);
        }
    }

    pub fn table_item_names(&self) -> Vec<String> {
        if let Some(surface) = self.table_surface() {
            return surface
                .items
                .iter()
                .map(|i| i.name().to_string())
                .collect();
        }
        self.cabin_state()
            .map(|c| c.table_item_names())
            .unwrap_or_default()
    }

    /// Create a new game state with initial values
    pub fn new(map: &WorldMap) -> Self {
        let mut rng = rand::thread_rng();
        let mut state = Self {
            version: "1.0".to_string(),
            time: WorldTime::new(),
            weather: RegionalWeather::new(),
            player: Player::new(),
            wildlife: spawn_wildlife(),
            objects: ObjectRegistry::new(),
            custom_names: HashMap::new(),
            pending_messages: Vec::new(),
            legacy_cabin: None,
            legacy_wood_shed: None,
            legacy_trees: None,
        };
        state.bootstrap_structures();
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
                    if state.wildlife.is_empty() {
                        tracing::info!("Wildlife was empty, spawning new wildlife");
                        state.wildlife = spawn_wildlife();
                    }
                    if state.custom_names.is_empty() {
                        state.custom_names = HashMap::new();
                    }

                    state.ensure_tree_objects_from_legacy();
                    state.bootstrap_structures();

                    let mut rng = rand::thread_rng();
                    state.seed_tree_population(map, &mut rng, 10);
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
        if let Some(cabin) = self.cabin_state_mut() {
            if let Some(fire_msg) = cabin.fireplace.update() {
                self.pending_messages.push(fire_msg);
            }
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
        let fire_heat = if matches!(self.player.room, Some(Room::CabinMain)) {
            self.cabin_state()
                .map(|c| c.fireplace.heat_output())
                .unwrap_or(0.0)
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
        self.objects.living_tree_count()
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
            if self
                .objects
                .objects_at(&pos)
                .iter()
                .any(|p| matches!(p.object.kind, ObjectKind::Tree(_)) || p.object.anchored)
            {
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
        let mut tree = Tree::with_random_fruit(pos, kind, rng);
        tree.apply_kind_defaults();
        let id = format!("tree-{}-{}-{}", pos.row, pos.col, self.objects.placed.len());
        self.objects
            .add(id, pos, WorldObject::new(ObjectKind::Tree(tree)));
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
        self.objects
            .for_each_tree_mut(|tree, _| tree.tick_growth(rng));
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
            let in_cabin = self
                .cabin_state()
                .map(|c| c.items.contains(item) || c.table_items.contains(item))
                .unwrap_or(false);
            let on_table = self
                .table_surface()
                .map(|s| s.items.contains(item))
                .unwrap_or(false);
            if in_cabin || on_table {
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
