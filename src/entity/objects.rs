use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    // Tools
    Axe,
    Matchbox,
    FishingRod,

    // Resources
    Log,           // Unsplit log
    Firewood,      // Split firewood
    Kindling,      // Fire starter
    LitKindling,   // Burning kindling

    // Found items
    Pinecone,
    Mushroom,
    WildBerry,
    Feather,
    Stone,
    Driftwood,

    // Cabin items
    OldBook,
    StrangeCompass,
    AncientMap,
    TeaCup,
    WoolBlanket,
}

impl Item {
    pub fn name(&self) -> &'static str {
        match self {
            Item::Axe => "axe",
            Item::Matchbox => "matchbox",
            Item::FishingRod => "fishing rod",
            Item::Log => "log",
            Item::Firewood => "firewood",
            Item::Kindling => "kindling",
            Item::LitKindling => "lit kindling",
            Item::Pinecone => "pinecone",
            Item::Mushroom => "mushroom",
            Item::WildBerry => "wild berry",
            Item::Feather => "feather",
            Item::Stone => "smooth stone",
            Item::Driftwood => "driftwood",
            Item::OldBook => "old leather-bound book",
            Item::StrangeCompass => "strange compass",
            Item::AncientMap => "ancient map",
            Item::TeaCup => "ceramic tea cup",
            Item::WoolBlanket => "wool blanket",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Item::Axe => "A sturdy woodcutting axe with a worn hickory handle. The blade shows signs of use but remains sharp.",
            Item::Matchbox => "A small wooden box containing matches. The striker strip is well-worn but functional.",
            Item::FishingRod => "A simple bamboo fishing rod with a length of sturdy line.",
            Item::Log => "A thick, unsplit log. Too large to burn directly - needs to be chopped into firewood.",
            Item::Firewood => "A piece of split firewood, perfect for the fireplace.",
            Item::Kindling => "A bundle of dry twigs and wood shavings - ideal for starting a fire.",
            Item::LitKindling => "Flames dance across the burning kindling. Use it quickly before it burns out!",
            Item::Pinecone => "A perfect pinecone, scales tightly closed. It smells faintly of resin.",
            Item::Mushroom => "A wild mushroom with a russet cap. Beautiful, but best left uneaten unless identified.",
            Item::WildBerry => "A handful of wild berries. They look safe to eat.",
            Item::Feather => "A beautiful feather, iridescent in the light.",
            Item::Stone => "A smooth, flat stone - perfect for skipping across water.",
            Item::Driftwood => "A piece of pale driftwood, smoothed by water and time.",
            Item::OldBook => "An ancient tome filled with strange symbols and illustrations of stars and geometry.",
            Item::StrangeCompass => "This compass needle doesn't point north - it seems to follow something else entirely.",
            Item::AncientMap => "A yellowed map showing lands that don't match any known geography.",
            Item::TeaCup => "A delicate ceramic cup with a faded floral pattern. It has seen many quiet mornings.",
            Item::WoolBlanket => "A thick, warm wool blanket. It carries the faint scent of woodsmoke.",
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            Item::Axe => 3.0,
            Item::Matchbox => 0.1,
            Item::FishingRod => 1.0,
            Item::Log => 5.0,
            Item::Firewood => 1.5,
            Item::Kindling => 0.3,
            Item::LitKindling => 0.3,
            Item::Pinecone => 0.1,
            Item::Mushroom => 0.1,
            Item::WildBerry => 0.05,
            Item::Feather => 0.01,
            Item::Stone => 0.3,
            Item::Driftwood => 0.5,
            Item::OldBook => 1.0,
            Item::StrangeCompass => 0.2,
            Item::AncientMap => 0.1,
            Item::TeaCup => 0.2,
            Item::WoolBlanket => 2.0,
        }
    }

    pub fn fuel_value(&self) -> Option<f32> {
        match self {
            Item::Firewood => Some(30.0),
            Item::Kindling => Some(10.0),
            Item::Log => Some(60.0), // If you could somehow burn it whole
            Item::Pinecone => Some(5.0),
            Item::Driftwood => Some(15.0),
            Item::OldBook => Some(8.0), // You monster
            _ => None,
        }
    }

    pub fn from_str(s: &str) -> Option<Item> {
        let normalized = s.to_lowercase();
        match normalized.as_str() {
            "axe" => Some(Item::Axe),
            "matchbox" | "matches" => Some(Item::Matchbox),
            "fishing rod" | "rod" => Some(Item::FishingRod),
            "log" => Some(Item::Log),
            "firewood" | "wood" => Some(Item::Firewood),
            "kindling" => Some(Item::Kindling),
            "lit kindling" | "burning kindling" => Some(Item::LitKindling),
            "pinecone" | "pine cone" => Some(Item::Pinecone),
            "mushroom" => Some(Item::Mushroom),
            "berry" | "berries" | "wild berry" | "wild berries" => Some(Item::WildBerry),
            "feather" => Some(Item::Feather),
            "stone" | "rock" => Some(Item::Stone),
            "driftwood" => Some(Item::Driftwood),
            "book" | "old book" => Some(Item::OldBook),
            "compass" | "strange compass" => Some(Item::StrangeCompass),
            "map" | "ancient map" => Some(Item::AncientMap),
            "cup" | "tea cup" | "teacup" => Some(Item::TeaCup),
            "blanket" | "wool blanket" => Some(Item::WoolBlanket),
            _ => None,
        }
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

    /// Fuel consumption per tick
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
}

impl Fireplace {
    pub fn new() -> Self {
        Self {
            state: FireState::Cold,
            fuel: 0.0,
        }
    }

    pub fn add_fuel(&mut self, fuel_value: f32) {
        self.fuel += fuel_value;
        self.update_state();
    }

    pub fn ignite(&mut self) -> bool {
        if self.fuel < 5.0 {
            return false; // Not enough fuel
        }
        if self.state == FireState::Cold {
            self.state = FireState::Smoldering;
            true
        } else {
            false // Already lit
        }
    }

    pub fn update(&mut self) {
        // Consume fuel
        let consumption = self.state.fuel_consumption();
        self.fuel = (self.fuel - consumption).max(0.0);

        // Update state based on fuel level
        self.update_state();
    }

    fn update_state(&mut self) {
        self.state = match self.fuel {
            f if f <= 0.0 => FireState::Cold,
            f if f < 10.0 => {
                // Can only be Smoldering if already lit
                if self.state != FireState::Cold { FireState::Smoldering } else { FireState::Cold }
            }
            f if f < 40.0 => {
                if self.state != FireState::Cold { FireState::Burning } else { FireState::Cold }
            }
            _ => {
                if self.state != FireState::Cold { FireState::Roaring } else { FireState::Cold }
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

    pub fn place_log(&mut self) -> bool {
        if self.has_log {
            false
        } else {
            self.has_log = true;
            true
        }
    }

    pub fn remove_log(&mut self) -> bool {
        if self.has_log {
            self.has_log = false;
            true
        } else {
            false
        }
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
                Item::StrangeCompass,
                Item::AncientMap,
                Item::TeaCup,
                Item::WoolBlanket,
            ],
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
}

impl Default for Cabin {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WoodShed {
    pub logs: u32,         // Unsplit logs available
    pub firewood: u32,     // Already split firewood
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

/// Location-specific items that can be found/foraged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationItems {
    pub items: Vec<(Item, u32)>, // Item and quantity
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
        self.items.iter()
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
