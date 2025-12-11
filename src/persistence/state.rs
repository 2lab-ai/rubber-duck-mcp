use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::entity::*;
use crate::world::*;
use rand::Rng;

const TUTORIAL_BOOK_ID: &str = "book-tutorial";
const OLD_BOOK_ID: &str = "book-old";
const DEATH_NOTE_ID: &str = "book-death-note";
const FISHING_BOOK_ID: &str = "book-fishing";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForageNode {
    pub charges: u8,
    pub cooldown: u8,
}

impl ForageNode {
    pub fn new(biome: Biome, rng: &mut impl Rng) -> Self {
        let charges = match biome {
            Biome::Desert => rng.gen_range(1..=2),
            Biome::Oasis => rng.gen_range(3..=4),
            Biome::WinterForest => rng.gen_range(2..=3),
            Biome::Lake => rng.gen_range(3..=5),
            Biome::BambooGrove => rng.gen_range(3..=5),
            _ => rng.gen_range(4..=6),
        };
        Self {
            charges,
            cooldown: 0,
        }
    }

    pub fn tick(&mut self, biome: Biome, rng: &mut impl Rng) {
        if self.charges > 0 {
            return;
        }
        if self.cooldown > 0 {
            self.cooldown -= 1;
            if self.cooldown == 0 {
                *self = Self::new(biome, rng);
            }
        }
    }
}

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
    #[serde(default)]
    pub forage_nodes: HashMap<Position, ForageNode>,
    #[serde(default = "GameState::default_books")]
    pub books: HashMap<String, BookEntry>,
    #[serde(default = "GameState::default_next_book_id")]
    pub next_book_id: u32,
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
    pub fn default_books() -> HashMap<String, BookEntry> {
        HashMap::new()
    }

    pub fn default_next_book_id() -> u32 {
        1
    }

    pub fn cabin_state(&self) -> Option<&Cabin> {
        self.objects.find("cabin").and_then(|p| p.object.as_cabin())
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
            .add("cabin_table", Position::new(0, 0), table_obj);
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

    fn ensure_player_visit(&mut self) {
        self.player.mark_visited();
    }

    pub fn damage_tool(&mut self, item: &Item, amount: u32, context: &str) {
        let Some(max) = Player::tool_max_durability(item) else {
            return;
        };
        let entry = self.player.tool_durability.entry(*item).or_insert(max);
        if *entry <= amount {
            let _ = self.player.inventory.remove(item, 1);
            self.player.tool_durability.remove(item);
            self.pending_messages
                .push(format!("Your {} breaks while {}.", item.name(), context));
        } else {
            *entry -= amount;
        }
    }

    pub fn refresh_blueprint_knowledge(&mut self, push_messages: bool) {
        let tutorial_done = self.book_completed(TUTORIAL_BOOK_ID);
        let fishing_done = self.book_completed(FISHING_BOOK_ID);
        let active_target = self.player.active_project.as_ref().map(|bp| bp.target_item);

        let add_if = |state: &mut Self, item: Item, condition: bool, reason: &str| {
            if !(condition || active_target == Some(item)) {
                return;
            }
            if state.player.known_blueprints.insert(item) && push_messages {
                state.pending_messages.push(format!(
                    "You learned the {} blueprint. {}",
                    item.name(),
                    reason
                ));
            }
        };

        add_if(
            self,
            Item::StoneKnife,
            self.player.skills.survival >= 8,
            "Basic survival practice reveals how to knap and lash a knife.",
        );
        add_if(
            self,
            Item::Cordage,
            self.player.skills.tailoring >= 8,
            "You recognize how to twist plant fibers into rope.",
        );
        add_if(
            self,
            Item::Campfire,
            self.player.skills.fire_making >= 8 || self.player.skills.survival >= 8,
            "Fire-making fundamentals click into place.",
        );
        add_if(
            self,
            Item::StoneAxe,
            self.player.skills.woodcutting >= 12 || tutorial_done,
            "Woodcutting skill or completing the cabin tutorial reveals axe joinery.",
        );
        add_if(
            self,
            Item::FishingRod,
            fishing_done,
            "Finishing the Book of Fishing shows how to lash a simple rod.",
        );
    }

    fn ensure_book_registry(&mut self) {
        let mut insert_if_missing = |id: &str, title: &str, pages: Vec<&str>, writable: bool| {
            if !self.books.contains_key(id) {
                self.books.insert(
                    id.to_string(),
                    BookEntry {
                        id: id.to_string(),
                        title: title.to_string(),
                        pages: pages.into_iter().map(|p| p.to_string()).collect(),
                        writable,
                    },
                );
            }
        };

        insert_if_missing(
            TUTORIAL_BOOK_ID,
            "Cabin Tutorial",
            vec![
                "Welcome. Use 'use hands on bush' to forage, 'create campfire' to plan a fire, and 'use log on blueprint' to build.",
                "Write a title on a blank book: write 제목:My Journal on 빈 책. Then write pages: write 페이지1:Hello on book-{id}.",
            ],
            false,
        );
        insert_if_missing(
            OLD_BOOK_ID,
            "Weathered Journal",
            vec![
                "The cabin creaks but endures. The lake stays still even in wind.",
                "Someone underlined a phrase: 'Keep writing; the ink remembers what you might forget.'",
            ],
            false,
        );
        insert_if_missing(
            DEATH_NOTE_ID,
            "Death Note",
            vec!["The human whose name is written in this note shall die."],
            true,
        );
        insert_if_missing(
            FISHING_BOOK_ID,
            "Book of Fishing",
            vec![
                "A simple rod needs a straight pole, a bendable tip, and cordage tied in clean knots. Bamboo or a stiff stick will do.",
                "Fish cruise the shallows at dawn and dusk. In storms they sink deep and hide; in clear weather, keep quiet and watch for ripples.",
                "Close the book and you can almost feel the rhythm of casting. You think you could craft a wooden fishing rod now.",
            ],
            false,
        );

        let max_seen = self
            .books
            .keys()
            .filter_map(|k| k.strip_prefix("book-"))
            .filter_map(|n| n.parse::<u32>().ok())
            .max()
            .unwrap_or(0);
        if self.next_book_id <= max_seen {
            self.next_book_id = max_seen + 1;
        }
    }

    fn ensure_cabin_books(&mut self) {
        let Some(cabin) = self.cabin_state_mut() else {
            return;
        };
        let ensure = |cabin: &mut Cabin, id: &str, item: Item| {
            if !cabin.book_ids.iter().any(|b| b == id) {
                cabin.book_ids.push(id.to_string());
            }
            if !cabin.items.contains(&item) {
                cabin.items.push(item);
            }
        };
        ensure(cabin, TUTORIAL_BOOK_ID, Item::TutorialBook);
        ensure(cabin, OLD_BOOK_ID, Item::OldBook);
        ensure(cabin, DEATH_NOTE_ID, Item::DeathNote);
        ensure(cabin, FISHING_BOOK_ID, Item::BookOfFishing);
    }

    pub fn generate_book_id(&mut self) -> String {
        let id = format!("book-{}", self.next_book_id);
        self.next_book_id += 1;
        id
    }

    pub fn book_entry(&self, id: &str) -> Option<&BookEntry> {
        self.books.get(id)
    }

    pub fn book_entry_mut(&mut self, id: &str) -> Option<&mut BookEntry> {
        self.books.get_mut(id)
    }

    pub fn register_book(&mut self, entry: BookEntry) -> String {
        let id = entry.id.clone();
        self.books.insert(id.clone(), entry);
        id
    }

    pub fn player_has_book(&self, id: &str) -> bool {
        self.player.book_ids.iter().any(|b| b == id)
    }

    pub fn add_player_book(&mut self, id: &str) {
        if !self.player.book_ids.iter().any(|b| b == id) {
            self.player.book_ids.push(id.to_string());
        }
    }

    pub fn remove_player_book(&mut self, id: &str) -> bool {
        if let Some(pos) = self.player.book_ids.iter().position(|b| b == id) {
            self.player.book_ids.remove(pos);
            return true;
        }
        false
    }

    pub fn pop_any_player_book(&mut self) -> Option<String> {
        self.player.book_ids.pop()
    }

    pub fn book_id_for_item<'a>(&self, item: &'a Item) -> Option<&'a str> {
        match item {
            Item::TutorialBook => Some(TUTORIAL_BOOK_ID),
            Item::OldBook => Some(OLD_BOOK_ID),
            Item::DeathNote => Some(DEATH_NOTE_ID),
            Item::BookOfFishing => Some(FISHING_BOOK_ID),
            _ => None,
        }
    }

    pub fn take_cabin_book_for_item(&mut self, item: &Item) -> Option<String> {
        let id_hint = self.book_id_for_item(item).map(|s| s.to_string());
        let Some(cabin) = self.cabin_state_mut() else {
            return None;
        };
        if let Some(id) = id_hint {
            if let Some(pos) = cabin.book_ids.iter().position(|b| b == &id) {
                return Some(cabin.book_ids.remove(pos));
            }
        }
        if matches!(item, Item::Book) {
            return cabin.book_ids.pop();
        }
        None
    }

    pub fn add_cabin_book(&mut self, id: String) {
        if let Some(cabin) = self.cabin_state_mut() {
            if !cabin.book_ids.iter().any(|b| b == &id) {
                cabin.book_ids.push(id);
            }
        }
    }

    pub fn accessible_book(&self, query: &str) -> Option<&BookEntry> {
        let q = query.to_lowercase();
        let mut ids_to_check: Vec<String> = self.player.book_ids.clone();
        if matches!(self.player.room, Some(Room::CabinMain)) {
            if let Some(cabin) = self.cabin_state() {
                ids_to_check.extend(cabin.book_ids.clone());
            }
        }
        for id in ids_to_check {
            if let Some(book) = self.books.get(&id) {
                if book.id.to_lowercase().contains(&q) || book.title.to_lowercase().contains(&q) {
                    return Some(book);
                }
            }
        }
        None
    }

    pub fn accessible_book_ids(&self) -> Vec<String> {
        let mut ids = self.player.book_ids.clone();
        if matches!(self.player.room, Some(Room::CabinMain)) {
            if let Some(cabin) = self.cabin_state() {
                ids.extend(cabin.book_ids.clone());
            }
        }
        ids
    }

    pub fn player_or_cabin_has_book(&self, id: &str) -> bool {
        self.player.book_ids.iter().any(|b| b == id)
            || (matches!(self.player.room, Some(Room::CabinMain))
                && self
                    .cabin_state()
                    .map(|c| c.book_ids.iter().any(|b| b == id))
                    .unwrap_or(false))
    }

    pub fn book_page(&self, id: &str) -> usize {
        self.player.book_progress.get(id).copied().unwrap_or(0)
    }

    pub fn set_book_page(&mut self, id: &str, page: usize) {
        self.player.book_progress.insert(id.to_string(), page);
    }

    fn book_completed(&self, id: &str) -> bool {
        let read_page = self.book_page(id);
        let total_pages = self.books.get(id).map(|b| b.pages.len()).unwrap_or(0);
        total_pages > 0 && read_page >= total_pages
    }

    pub fn knows_blueprint(&self, item: Item) -> bool {
        self.player.known_blueprints.contains(&item)
    }

    pub fn known_blueprint_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .player
            .known_blueprints
            .iter()
            .map(|i| i.name().to_string())
            .collect();
        names.sort();
        names
    }

    pub fn blueprint_hint_text(&self, item: Item) -> Option<&'static str> {
        self.blueprint_hint(item)
    }

    fn blueprint_hint(&self, item: Item) -> Option<&'static str> {
        match item {
            Item::StoneAxe => {
                Some("Raise woodcutting to 12 or finish the Cabin Tutorial to learn it.")
            }
            Item::StoneKnife => Some("Build basic survival skill to unlock this."),
            Item::Campfire => Some("Practice fire-making to level 8+ to learn this pattern."),
            Item::Cordage => Some("Tailoring 8+ reveals how to twist cordage."),
            Item::FishingRod => Some("Finish reading the Book of Fishing to unlock this."),
            _ => None,
        }
    }

    pub fn locked_blueprint_hints(&self) -> Vec<String> {
        let targets = [
            Item::StoneKnife,
            Item::Campfire,
            Item::Cordage,
            Item::StoneAxe,
            Item::FishingRod,
        ];
        let mut hints = Vec::new();
        for item in targets {
            if !self.knows_blueprint(item) {
                if let Some(hint) = self.blueprint_hint(item) {
                    hints.push(format!("{}: {}", item.name(), hint));
                }
            }
        }
        hints
    }

    pub fn foraging_node_for(
        &mut self,
        pos: Position,
        map: &WorldMap,
        rng: &mut impl Rng,
    ) -> &mut ForageNode {
        let biome = pos
            .as_usize()
            .and_then(|(r, c)| map.get_tile(r, c).map(|t| t.biome))
            .unwrap_or(Biome::MixedForest);
        self.forage_nodes
            .entry(pos)
            .or_insert_with(|| ForageNode::new(biome, rng))
    }

    pub fn on_player_pickup(&mut self, item: &Item) {
        if matches!(
            item,
            Item::Book | Item::TutorialBook | Item::OldBook | Item::DeathNote | Item::BookOfFishing
        ) {
            if let Some(book_id) = self
                .take_cabin_book_for_item(item)
                .or_else(|| self.book_id_for_item(item).map(|s| s.to_string()))
            {
                self.add_player_book(&book_id);
            }
        }
    }

    pub fn on_player_drop(&mut self, item: &Item) -> Option<String> {
        if matches!(
            item,
            Item::Book | Item::TutorialBook | Item::OldBook | Item::DeathNote | Item::BookOfFishing
        ) {
            // Prefer removing a matching special book id; otherwise pop any
            if let Some(id) = self
                .book_id_for_item(item)
                .and_then(|id| self.remove_player_book(id).then(|| id.to_string()))
            {
                return Some(id);
            }
            if let Some(id) = self.pop_any_player_book() {
                return Some(id);
            }
        }
        None
    }

    fn bootstrap_structures(&mut self) {
        let mut cabin_state = self.legacy_cabin.take().unwrap_or_else(Cabin::new);
        Self::ensure_core_cabin_items(&mut cabin_state);
        let mut table_items = std::mem::take(&mut cabin_state.table_items);

        if self.objects.find("cabin").is_none() {
            self.objects.add(
                "cabin",
                Position::new(0, 0),
                WorldObject::new(ObjectKind::Cabin(cabin_state)),
            );
        } else if let Some(po) = self.objects.find_mut("cabin") {
            if let Some(cabin) = po.object.as_cabin_mut() {
                Self::ensure_core_cabin_items(cabin);
                if table_items.is_empty() && !cabin.table_items.is_empty() {
                    table_items.extend(cabin.table_items.iter().copied());
                }
            }
            // Move cabin to new origin
            if po.position != Position::new(0, 0) {
                po.position = Position::new(0, 0);
            }
        }

        let wood_shed_state = self.legacy_wood_shed.take().unwrap_or_else(WoodShed::new);
        if self.objects.find("wood_shed").is_none() {
            self.objects.add(
                "wood_shed",
                Position::new(-1, -1),
                WorldObject::new(ObjectKind::WoodShed(wood_shed_state)),
            );
        } else if let Some(po) = self.objects.find_mut("wood_shed") {
            if po.object.as_wood_shed().is_none() {
                po.object.kind = ObjectKind::WoodShed(wood_shed_state);
            }
            if po.position != Position::new(-1, -1) {
                po.position = Position::new(-1, -1);
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
            return surface.items.iter().map(|i| i.name().to_string()).collect();
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
            forage_nodes: HashMap::new(),
            books: GameState::default_books(),
            next_book_id: GameState::default_next_book_id(),
            pending_messages: Vec::new(),
            legacy_cabin: None,
            legacy_wood_shed: None,
            legacy_trees: None,
        };
        state.ensure_book_registry();
        state.bootstrap_structures();
        state.ensure_cabin_books();
        state.ensure_player_visit();
        state.refresh_blueprint_knowledge(false);
        state.seed_bamboo_grove();
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
                    if state.forage_nodes.is_empty() {
                        state.forage_nodes = HashMap::new();
                    }

                    if state.books.is_empty() {
                        state.books = GameState::default_books();
                    }
                    state.ensure_book_registry();

                    state.ensure_tree_objects_from_legacy();
                    state.bootstrap_structures();
                    state.ensure_cabin_books();
                    state.ensure_player_visit();
                    state.refresh_blueprint_knowledge(false);
                    state.seed_bamboo_grove();

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
            w.update(tod, map, &self.weather);
        }

        // Update fireplace and collect any warnings
        if let Some(cabin) = self.cabin_state_mut() {
            if let Some(fire_msg) = cabin.fireplace.update() {
                self.pending_messages.push(fire_msg);
            }
        }

        let mut rng = rand::thread_rng();
        self.update_trees(map, &mut rng);
        self.update_forage_nodes(map, &mut rng);

        // Hunger / thirst decay
        self.player.modify_fullness(-0.5);
        self.player.modify_hydration(-0.5);
        if self.player.fullness < 20.0 {
            self.player.modify_energy(-1.0);
            self.player.modify_mood(-1.0);
            if self.player.fullness < 10.0 {
                self.pending_messages
                    .push("Your stomach growls painfully. You need to eat soon.".to_string());
            }
        }
        if self.player.hydration < 20.0 {
            self.player.modify_energy(-1.0);
            if self.player.hydration < 10.0 {
                self.player.modify_health(-0.5);
                self.pending_messages
                    .push("Your mouth is dry and head swims. Drink water soon.".to_string());
            }
        }

        // Update player warmth based on environment
        self.update_player_comfort(map);

        // Check for newly unlocked blueprints as skills/books progress
        self.refresh_blueprint_knowledge(true);
    }

    fn update_forage_nodes(&mut self, map: &WorldMap, rng: &mut impl Rng) {
        let positions: Vec<Position> = self.forage_nodes.keys().copied().collect();
        for pos in positions {
            if let Some(node) = self.forage_nodes.get_mut(&pos) {
                let biome = pos
                    .as_usize()
                    .and_then(|(r, c)| map.get_tile(r, c).map(|t| t.biome))
                    .unwrap_or(Biome::MixedForest);
                node.tick(biome, rng);
            }
        }
    }

    fn update_player_comfort(&mut self, map: &WorldMap) {
        let fire_heat = if matches!(self.player.room, Some(Room::CabinMain)) {
            self.cabin_state()
                .map(|c| c.fireplace.heat_output())
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Get position for temperature calculation
        let world_row = self.player.position.row;
        let world_col = self.player.position.col;
        let (row, col) = self
            .player
            .position
            .as_usize()
            .unwrap_or((MAP_ORIGIN_ROW as usize, MAP_ORIGIN_COL as usize));
        let biome = map
            .get_tile(row, col)
            .map(|t| t.biome)
            .unwrap_or(Biome::MixedForest);
        let tod = self.time.time_of_day();

        let base_temp = match self.player.room {
            Some(_) if fire_heat > 0.0 => 18.0 + fire_heat,
            Some(_) => 16.0, // Indoor base temp
            None => {
                let weather_temp = self
                    .weather
                    .get_for_position(world_row, world_col)
                    .temperature_modifier();
                biome.base_temperature() + tod.temperature_modifier() + weather_temp
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
            let row = rng.gen_range(-MAP_EXTENT..=MAP_EXTENT);
            let col = rng.gen_range(-MAP_EXTENT..=MAP_EXTENT);
            let pos = Position::new(row, col);
            if self
                .objects
                .objects_at(&pos)
                .iter()
                .any(|p| matches!(p.object.kind, ObjectKind::Tree(_)) || p.object.anchored)
            {
                continue;
            }
            let Some((gr, gc)) = pos.as_usize() else {
                continue;
            };
            let Some(tile) = map.get_tile(gr, gc) else {
                continue;
            };
            if matches!(tile.tile_type, TileType::Forest(_)) && tile.walkable {
                return Some(pos);
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
        let kind = pos
            .as_usize()
            .and_then(|(r, c)| map.get_tile(r, c))
            .map(|t| {
                if matches!(t.biome, Biome::BambooGrove) {
                    TreeType::Bamboo
                } else {
                    self.random_tree_kind(rng)
                }
            })
            .unwrap_or_else(|| self.random_tree_kind(rng));
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

    fn seed_bamboo_grove(&mut self) {
        let grove_positions = [
            Position::new(0, -2),
            Position::new(0, -3),
            Position::new(1, -2),
        ];
        for pos in grove_positions {
            if self
                .objects
                .objects_at(&pos)
                .iter()
                .any(|p| matches!(p.object.kind, ObjectKind::Tree(_)))
            {
                continue;
            }
            let mut tree = Tree::new(pos, TreeType::Bamboo);
            tree.apply_kind_defaults();
            let id = format!("bamboo-{}-{}", pos.row, pos.col);
            self.objects
                .add(id, pos, WorldObject::new(ObjectKind::Tree(tree)));
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
        self.custom_name(item)
            .map(|s| s.to_string())
            .unwrap_or_else(|| item.name().to_string())
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
        Self {
            map,
            state,
            state_path,
        }
    }

    pub fn save(&self) -> Result<()> {
        self.state.save(&self.state_path)
    }

    pub fn tick(&mut self) {
        self.state.tick_with_map(&self.map);
    }
}
