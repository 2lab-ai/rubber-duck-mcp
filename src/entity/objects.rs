use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Item {
    // Tools
    Axe,
    StoneAxe,   // Crafted axe
    Knife,      // Factory knife
    StoneKnife, // Crafted knife
    Matchbox,
    FishingRod,

    // Resources
    Log,         // Unsplit log
    Stick,       // Basic wood
    Firewood,    // Split firewood
    Kindling,    // Fire starter
    LitKindling, // Burning kindling
    Charcoal,    // From burnt wood
    Ash,         // Final residue

    // Crafting Materials
    Stone,
    SharpStone, // Knapped stone
    PlantFiber, // From bushes/grass
    Cordage,    // Rope from fiber
    Sap,        // Tree resin

    // Found items
    Pinecone,
    Mushroom,
    WildBerry,
    Feather,
    Driftwood,
    Bark,
    DryLeaves,
    WildHerbs,    // For tea
    Apple,        // Fruit from nearby trees
    Date,         // From oasis
    Bamboo,       // From bamboo grove
    Paper,        // Crafted from bamboo
    BlankBook,    // Unbound/writable book
    Book,         // Written book
    TutorialBook, // Tutorial copy
    DeathNote,    // Spooky notebook

    // Structures / Placeables (as items for blueprint targets)
    Campfire,

    // Cabin items
    OldBook,
    StrangeCompass,
    AncientMap,
    TeaCup,
    WoolBlanket,
    Kettle,         // For boiling water
    WaterKettle,    // Kettle with water
    HotWaterKettle, // Kettle with boiling water
    HerbalTea,      // Finished tea!
    RubberDuck,
}

const ALL_ITEMS: [Item; 45] = [
    Item::Axe,
    Item::StoneAxe,
    Item::Knife,
    Item::StoneKnife,
    Item::Matchbox,
    Item::FishingRod,
    Item::Log,
    Item::Stick,
    Item::Firewood,
    Item::Kindling,
    Item::LitKindling,
    Item::Charcoal,
    Item::Ash,
    Item::Stone,
    Item::SharpStone,
    Item::PlantFiber,
    Item::Cordage,
    Item::Sap,
    Item::Pinecone,
    Item::Mushroom,
    Item::WildBerry,
    Item::Feather,
    Item::Driftwood,
    Item::Bark,
    Item::DryLeaves,
    Item::WildHerbs,
    Item::Apple,
    Item::Date,
    Item::Bamboo,
    Item::Paper,
    Item::BlankBook,
    Item::Book,
    Item::TutorialBook,
    Item::DeathNote,
    Item::Campfire,
    Item::OldBook,
    Item::StrangeCompass,
    Item::AncientMap,
    Item::TeaCup,
    Item::WoolBlanket,
    Item::Kettle,
    Item::WaterKettle,
    Item::HotWaterKettle,
    Item::HerbalTea,
    Item::RubberDuck,
];

impl Item {
    pub fn name(&self) -> &'static str {
        match self {
            Item::Axe => "axe",
            Item::StoneAxe => "stone axe",
            Item::Knife => "knife",
            Item::StoneKnife => "stone knife",
            Item::Matchbox => "matchbox",
            Item::FishingRod => "fishing rod",
            Item::Log => "log",
            Item::Stick => "stick",
            Item::Firewood => "firewood",
            Item::Kindling => "kindling",
            Item::LitKindling => "lit kindling",
            Item::Charcoal => "charcoal",
            Item::Ash => "ash",
            Item::Stone => "stone",
            Item::SharpStone => "sharp stone",
            Item::PlantFiber => "plant fiber",
            Item::Cordage => "cordage",
            Item::Sap => "tree sap",
            Item::Pinecone => "pinecone",
            Item::Mushroom => "mushroom",
            Item::WildBerry => "wild berry",
            Item::Feather => "feather",
            Item::Driftwood => "driftwood",
            Item::Bark => "strip of bark",
            Item::DryLeaves => "dry leaves",
            Item::WildHerbs => "wild herbs",
            Item::Apple => "apple",
            Item::Date => "date",
            Item::Bamboo => "bamboo",
            Item::Paper => "paper",
            Item::BlankBook => "blank book",
            Item::Book => "book",
            Item::TutorialBook => "tutorial book",
            Item::DeathNote => "death note",
            Item::Campfire => "campfire",
            Item::OldBook => "old leather-bound book",
            Item::StrangeCompass => "strange compass",
            Item::AncientMap => "ancient map",
            Item::TeaCup => "ceramic tea cup",
            Item::WoolBlanket => "wool blanket",
            Item::Kettle => "copper kettle",
            Item::WaterKettle => "kettle with water",
            Item::HotWaterKettle => "kettle with hot water",
            Item::HerbalTea => "cup of herbal tea",
            Item::RubberDuck => "rubber duck",
        }
    }

    pub fn aliases(&self) -> &'static [&'static str] {
        match self {
            Item::Axe => &["hatchet", "iron axe"],
            Item::StoneAxe => &["primitive axe", "hand axe"],
            Item::Knife => &["hunting knife", "steel knife"],
            Item::StoneKnife => &["flint knife", "shard"],
            Item::Matchbox => &["matches", "match box"],
            Item::FishingRod => &["rod", "fishing pole", "pole"],
            Item::Log => &["unsplit log", "wood"],
            Item::Stick => &["branch", "twig", "wood stick"],
            Item::Firewood => &["split firewood", "split wood"],
            Item::Kindling => &["tinder", "fire starter", "shavings"],
            Item::LitKindling => &["burning kindling", "lit tinder"],
            Item::Charcoal => &["coal", "burnt wood"],
            Item::Ash => &["ashes"],
            Item::Stone => &["rock", "pebble"],
            Item::SharpStone => &["sharp rock", "flint flake"],
            Item::PlantFiber => &["fiber", "grass", "straw"],
            Item::Cordage => &["rope", "string", "twine"],
            Item::Sap => &["resin", "pitch"],
            Item::Pinecone => &["pine cone"],
            Item::Mushroom => &["fungus", "toadstool"],
            Item::WildBerry => &["berries", "wild berries", "berry"],
            Item::Feather => &["bird feather"],
            Item::Driftwood => &["drift wood"],
            Item::Bark => &["bark", "tree bark", "birch bark"],
            Item::DryLeaves => &["leaves", "leaf bundle"],
            Item::WildHerbs => &["herbs", "wild herbs"],
            Item::Apple => &["fruit", "red apple"],
            Item::Date => &["palm fruit"],
            Item::Bamboo => &["bamboo stalk", "stalk", "canebamboo"],
            Item::Paper => &["sheet", "paper sheet"],
            Item::BlankBook => &["blank book", "empty book", "notebook blank"],
            Item::Book => &["notebook", "journal", "book"],
            Item::TutorialBook => &["tutorial", "guide book", "tutorial book"],
            Item::DeathNote => &["death note", "black notebook", "cursed book"],
            Item::Campfire => &["fire", "fire pit"],
            Item::OldBook => &["book", "old book", "leather-bound book"],
            Item::StrangeCompass => &["compass", "odd compass"],
            Item::AncientMap => &["map", "weathered map", "old map"],
            Item::TeaCup => &["cup", "tea cup", "teacup", "ceramic cup"],
            Item::WoolBlanket => &["blanket", "warm blanket", "wool blanket"],
            Item::Kettle => &["copper kettle", "empty kettle"],
            Item::WaterKettle => &["water kettle", "kettle with water", "filled kettle"],
            Item::HotWaterKettle => &[
                "hot water",
                "hot kettle",
                "hot water kettle",
                "boiling water",
            ],
            Item::HerbalTea => &["tea", "herbal tea", "cup of tea"],
            Item::RubberDuck => &["duck", "yellow duck", "rubber ducky", "sage"],
        }
    }

    fn candidate_names(&self) -> impl Iterator<Item = &'static str> {
        std::iter::once(self.name()).chain(self.aliases().iter().copied())
    }

    pub fn matches_exact(&self, query: &str) -> bool {
        self.candidate_names()
            .any(|name| name.eq_ignore_ascii_case(query))
    }

    pub fn matches_suffix(&self, query: &str) -> bool {
        if query.len() < 3 {
            return false;
        }
        let q = query.to_lowercase();
        self.candidate_names().any(|name| {
            let n = name.to_lowercase();
            n.ends_with(&q) || q.ends_with(&n)
        })
    }

    pub fn description(&self) -> &'static str {
        match self {
            Item::Axe => "A sturdy woodcutting axe with a worn hickory handle.",
            Item::StoneAxe => "A crude axe made by tying a sharp stone to a stick.",
            Item::Knife => "A sharp hunting knife, essential for survival.",
            Item::StoneKnife => "A rough blade knapped from stone. Sharp enough to cut.",
            Item::Stick => "A sturdy stick. Useful for crafting or as fuel.",
            Item::Stone => "A smooth stone. Could be knapped into a tool.",
            Item::SharpStone => "A stone with a razor-sharp edge.",
            Item::PlantFiber => "Tough plant fibers gathered from bushes.",
            Item::Cordage => "A crude rope braided from plant fibers.",
            Item::Campfire => "A ring of stones with wood, ready to be lit.",
            Item::Bamboo => "A straight, light bamboo stalk harvested near the lake.",
            Item::Paper => "A thin sheet of paper made from bamboo pulp.",
            Item::BlankBook => "A blank book with crisp pages, ready for a title.",
            Item::Book => "A bound book. Check its ID to read or write pages.",
            Item::TutorialBook => "A short guide left in the cabin.",
            Item::DeathNote => "A black notebook with ominous weight.",
            _ => "A useful item.",
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            Item::Log => 5.0,
            Item::Stone => 0.5,
            Item::Axe => 3.0,
            Item::Bamboo => 1.0,
            Item::Paper => 0.05,
            Item::BlankBook => 0.3,
            Item::Book | Item::TutorialBook | Item::DeathNote | Item::OldBook => 0.4,
            _ => 0.1,
        }
    }

    pub fn fuel_value(&self) -> Option<f32> {
        match self {
            Item::Firewood => Some(30.0),
            Item::Kindling => Some(10.0),
            Item::LitKindling => Some(10.0),
            Item::Log => Some(60.0),
            Item::Stick => Some(5.0),
            Item::Pinecone => Some(5.0),
            Item::Bamboo => Some(8.0),
            Item::Paper => Some(1.0),
            Item::Bark => Some(6.0),
            Item::DryLeaves => Some(3.0),
            Item::Charcoal => Some(40.0),
            Item::OldBook => Some(8.0),
            _ => None,
        }
    }

    pub fn is_flammable(&self) -> bool {
        self.fuel_value().is_some()
    }

    pub fn is_tinder(&self) -> bool {
        matches!(
            self,
            Item::Kindling
                | Item::LitKindling
                | Item::Pinecone
                | Item::Bark
                | Item::DryLeaves
                | Item::OldBook
                | Item::Paper
                | Item::PlantFiber
        )
    }

    pub fn from_str(s: &str) -> Option<Item> {
        let query = s.trim();
        if query.is_empty() {
            return None;
        }
        for item in ALL_ITEMS.iter().copied() {
            if item.matches_exact(query) {
                return Some(item);
            }
        }
        for item in ALL_ITEMS.iter().copied() {
            if item.matches_suffix(query) {
                return Some(item);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FireState {
    Cold,
    Smoldering,
    Burning,
    Roaring,
}

impl FireState {
    pub fn name(&self) -> &'static str {
        match self {
            FireState::Cold => "cold and empty",
            FireState::Smoldering => "smoldering with weak flames",
            FireState::Burning => "burning steadily",
            FireState::Roaring => "roaring with powerful flames",
        }
    }

    pub fn heat_output(&self) -> f32 {
        match self {
            FireState::Cold => 0.0,
            FireState::Smoldering => 5.0,
            FireState::Burning => 15.0,
            FireState::Roaring => 25.0,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FireState::Cold => "The fireplace is cold and dark, filled only with old ash.",
            FireState::Smoldering => "Weak flames flicker among the embers, struggling to catch. A thin wisp of smoke rises.",
            FireState::Burning => "A healthy fire crackles in the hearth, casting dancing shadows on the walls. The warmth is welcoming.",
            FireState::Roaring => "The fire roars and pops, flames leaping high. The heat is intense, pushing back the cold completely.",
        }
    }

    pub fn fuel_consumption(&self) -> f32 {
        match self {
            FireState::Cold => 0.0,
            FireState::Smoldering => 1.0,
            FireState::Burning => 3.0,
            FireState::Roaring => 6.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fireplace {
    pub state: FireState,
    pub fuel: f32,
    pub tinder_ready: bool,
}

impl Fireplace {
    pub fn new() -> Self {
        Self {
            state: FireState::Cold,
            fuel: 0.0,
            tinder_ready: false,
        }
    }

    pub fn add_fuel_item(&mut self, item: Item) -> bool {
        if let Some(value) = item.fuel_value() {
            self.fuel += value;
            if item.is_tinder() {
                self.tinder_ready = true;
            }
            self.update_state();
            true
        } else {
            false
        }
    }

    pub fn ignite(&mut self) -> bool {
        if self.state != FireState::Cold || self.fuel < 5.0 || !self.tinder_ready {
            return false;
        }
        self.state = FireState::Smoldering;
        self.tinder_ready = false;
        true
    }

    pub fn clear_tinder(&mut self) {
        self.tinder_ready = false;
    }

    pub fn update(&mut self) -> Option<String> {
        let consumption = self.state.fuel_consumption();
        if consumption > 0.0 {
            self.fuel = (self.fuel - consumption).max(0.0);
        }

        let prev_state = self.state;
        self.update_state();

        if prev_state != FireState::Cold && self.state == FireState::Cold {
            Some("The fire dies down, leaving only faint wisps of smoke.".to_string())
        } else {
            None
        }
    }

    fn update_state(&mut self) {
        self.state = match self.fuel {
            f if f <= 0.0 => FireState::Cold,
            f if f < 10.0 => {
                if self.state == FireState::Cold {
                    FireState::Cold
                } else {
                    FireState::Smoldering
                }
            }
            f if f < 40.0 => {
                if self.state == FireState::Cold {
                    FireState::Cold
                } else {
                    FireState::Burning
                }
            }
            _ => {
                if self.state == FireState::Cold {
                    FireState::Cold
                } else {
                    FireState::Roaring
                }
            }
        };
    }

    pub fn heat_output(&self) -> f32 {
        self.state.heat_output()
    }
}

impl Default for Fireplace {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoppingBlock {
    pub has_log: bool,
}

impl ChoppingBlock {
    pub fn new() -> Self {
        Self { has_log: false }
    }
}

impl Default for ChoppingBlock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cabin {
    pub door_open: bool,
    pub fireplace: Fireplace,
    pub items: Vec<Item>,
    #[serde(default)]
    pub table_items: Vec<Item>,
    #[serde(default)]
    pub book_ids: Vec<String>,
}

impl Cabin {
    pub fn new() -> Self {
        Self {
            door_open: false,
            fireplace: Fireplace::new(),
            items: vec![
                Item::Matchbox,
                Item::Kindling,
                Item::Kindling,
                Item::OldBook,
                Item::TutorialBook,
                Item::DeathNote,
                Item::StrangeCompass,
                Item::AncientMap,
                Item::TeaCup,
                Item::WoolBlanket,
                Item::Kettle,
                Item::WildHerbs,
            ],
            table_items: vec![Item::RubberDuck],
            book_ids: Vec::new(),
        }
    }

    pub fn has_item(&self, item: &Item) -> bool {
        self.items.contains(item)
    }

    pub fn take_item(&mut self, item: &Item) -> bool {
        if let Some(idx) = self.items.iter().position(|i| i == item) {
            self.items.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn add_table_item(&mut self, item: Item) {
        self.table_items.push(item);
    }

    pub fn take_table_item(&mut self, item: &Item) -> bool {
        if let Some(idx) = self.table_items.iter().position(|i| i == item) {
            self.table_items.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn table_item_names(&self) -> Vec<String> {
        self.table_items
            .iter()
            .map(|i| i.name().to_string())
            .collect()
    }
}

impl Default for Cabin {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WoodShed {
    pub logs: u32,
    pub firewood: u32,
    pub axe_on_floor: bool,
    pub chopping_block: ChoppingBlock,
}

impl WoodShed {
    pub fn new() -> Self {
        Self {
            logs: 6,
            firewood: 0,
            axe_on_floor: true,
            chopping_block: ChoppingBlock::new(),
        }
    }
}

impl Default for WoodShed {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationItems {
    pub items: Vec<(Item, u32)>,
}

impl LocationItems {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: Item, qty: u32) {
        if let Some((_, existing)) = self.items.iter_mut().find(|(i, _)| i == &item) {
            *existing += qty;
        } else {
            self.items.push((item, qty));
        }
    }

    pub fn take(&mut self, item: &Item) -> bool {
        if let Some(idx) = self.items.iter().position(|(i, q)| i == item && *q > 0) {
            self.items[idx].1 -= 1;
            if self.items[idx].1 == 0 {
                self.items.remove(idx);
            }
            true
        } else {
            false
        }
    }

    pub fn list(&self) -> Vec<&Item> {
        self.items
            .iter()
            .filter(|(_, q)| *q > 0)
            .map(|(i, _)| i)
            .collect()
    }
}

impl Default for LocationItems {
    fn default() -> Self {
        Self::new()
    }
}
