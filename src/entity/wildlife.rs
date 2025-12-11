use crate::entity::Body;
use crate::world::{Biome, Direction, Position, RegionalWeather, TimeOfDay, Weather, WorldMap};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Species {
    // Spring/Autumn (North)
    Deer,
    Rabbit,
    Squirrel,
    Songbird,
    Woodpecker,
    Fox,

    // Desert (West)
    DesertLizard,
    Scorpion,
    DesertFox,
    Hawk,
    Rattlesnake,

    // Winter (East)
    SnowFox,
    Owl,
    Wolf,
    Caribou,
    SnowHare,

    // Lake
    Duck,
    Fish,
    Heron,
    Frog,
    Dragonfly,

    // General
    Butterfly,
    Bee,
    Pig,
    // Extra common game wildlife (herbivores)
    Boar,
    Goat,
    Sheep,
    Cow,
    Horse,
    Moose,
    Elk,
    Antelope,
    Bison,
    Camel,

    // Extra common game wildlife (carnivores)
    Bear,
    Lynx,
    Cougar,
    Tiger,
    Hyena,

    // Companions
    Dog,
    Cat,
}

impl Species {
    pub fn name(&self) -> &'static str {
        match self {
            Species::Deer => "deer",
            Species::Rabbit => "rabbit",
            Species::Squirrel => "squirrel",
            Species::Songbird => "songbird",
            Species::Woodpecker => "woodpecker",
            Species::Fox => "fox",
            Species::DesertLizard => "desert lizard",
            Species::Scorpion => "scorpion",
            Species::DesertFox => "fennec fox",
            Species::Hawk => "hawk",
            Species::Rattlesnake => "rattlesnake",
            Species::SnowFox => "arctic fox",
            Species::Owl => "snowy owl",
            Species::Wolf => "wolf",
            Species::Caribou => "caribou",
            Species::SnowHare => "snowshoe hare",
            Species::Duck => "duck",
            Species::Fish => "fish",
            Species::Heron => "heron",
            Species::Frog => "frog",
            Species::Dragonfly => "dragonfly",
            Species::Butterfly => "butterfly",
            Species::Bee => "bee",
            Species::Pig => "pig",
            Species::Boar => "boar",
            Species::Goat => "goat",
            Species::Sheep => "sheep",
            Species::Cow => "cow",
            Species::Horse => "horse",
            Species::Moose => "moose",
            Species::Elk => "elk",
            Species::Antelope => "antelope",
            Species::Bison => "bison",
            Species::Camel => "camel",
            Species::Bear => "bear",
            Species::Lynx => "lynx",
            Species::Cougar => "cougar",
            Species::Tiger => "tiger",
            Species::Hyena => "hyena",
            Species::Dog => "dog",
            Species::Cat => "cat",
        }
    }

    pub fn native_biomes(&self) -> Vec<Biome> {
        match self {
            Species::Deer
            | Species::Rabbit
            | Species::Squirrel
            | Species::Songbird
            | Species::Woodpecker
            | Species::Fox => {
                vec![Biome::SpringForest, Biome::MixedForest]
            }
            Species::DesertLizard
            | Species::Scorpion
            | Species::DesertFox
            | Species::Hawk
            | Species::Rattlesnake => {
                vec![Biome::Desert, Biome::Oasis]
            }
            Species::SnowFox
            | Species::Owl
            | Species::Wolf
            | Species::Caribou
            | Species::SnowHare => {
                vec![Biome::WinterForest]
            }
            Species::Duck | Species::Fish | Species::Heron | Species::Frog | Species::Dragonfly => {
                vec![Biome::Lake, Biome::Oasis]
            }
            Species::Butterfly | Species::Bee => {
                vec![Biome::SpringForest, Biome::MixedForest, Biome::Oasis]
            }
            Species::Pig | Species::Boar | Species::Goat | Species::Sheep | Species::Cow => {
                vec![Biome::Path, Biome::Clearing, Biome::MixedForest]
            }
            Species::Horse | Species::Elk | Species::Bison | Species::Antelope => {
                vec![Biome::SpringForest, Biome::MixedForest]
            }
            Species::Moose => vec![Biome::WinterForest, Biome::MixedForest],
            Species::Camel => vec![Biome::Desert, Biome::Oasis],
            Species::Bear => vec![Biome::SpringForest, Biome::MixedForest, Biome::WinterForest],
            Species::Lynx | Species::Cougar | Species::Tiger | Species::Hyena => {
                vec![Biome::SpringForest, Biome::MixedForest]
            }
            Species::Dog | Species::Cat => vec![Biome::Path, Biome::Clearing, Biome::MixedForest],
        }
    }

    pub fn activity_schedule(&self) -> ActivitySchedule {
        match self {
            Species::Owl | Species::Wolf | Species::Scorpion => ActivitySchedule::Nocturnal,
            Species::Deer | Species::Rabbit | Species::Fox | Species::Pig => {
                ActivitySchedule::Crepuscular
            }
            _ => ActivitySchedule::Diurnal,
        }
    }

    pub fn is_predator(&self) -> bool {
        matches!(
            self,
            Species::Fox
                | Species::DesertFox
                | Species::SnowFox
                | Species::Hawk
                | Species::Wolf
                | Species::Owl
                | Species::Rattlesnake
                | Species::Scorpion
                | Species::Heron
                | Species::Bear
                | Species::Lynx
                | Species::Cougar
                | Species::Tiger
                | Species::Hyena
                | Species::Dog
                | Species::Cat
        )
    }

    /// Generate a description snippet for this animal doing an action
    pub fn describe_action(&self, behavior: Behavior) -> String {
        let name = self.name();
        match (self, behavior) {
            (Species::Deer, Behavior::Grazing) => {
                format!("A {} grazes peacefully on tender grass.", name)
            }
            (Species::Deer, Behavior::Alert) => format!(
                "A {} stands frozen, ears twitching at some distant sound.",
                name
            ),
            (Species::Deer, Behavior::Moving) => {
                format!("A {} bounds gracefully between the trees.", name)
            }

            (Species::Rabbit, Behavior::Foraging) => {
                format!("A {} nibbles on clover, nose twitching constantly.", name)
            }
            (Species::Rabbit, Behavior::Alert) => {
                format!("A {} sits upright, scanning for danger.", name)
            }

            (Species::Squirrel, Behavior::Foraging) => {
                format!("A {} busily gathers acorns, stuffing its cheeks.", name)
            }
            (Species::Squirrel, Behavior::Moving) => {
                format!("A {} scampers up a tree trunk in spiraling leaps.", name)
            }

            (Species::Songbird, Behavior::Singing) => {
                format!("A {} trills a beautiful melody from the branches.", name)
            }
            (Species::Songbird, Behavior::Moving) => {
                format!("A {} flutters between branches.", name)
            }

            (Species::Woodpecker, _) => {
                format!("A {} drums rhythmically against a tree trunk.", name)
            }

            (Species::Fox, Behavior::Hunting) => format!(
                "A {} stalks through the underbrush, focused and silent.",
                name
            ),
            (Species::Fox, Behavior::Resting) => format!(
                "A {} curls up in a sunny patch, tail wrapped around its nose.",
                name
            ),

            (Species::DesertLizard, Behavior::Basking) => {
                format!("A {} basks on a sun-warmed rock.", name)
            }
            (Species::DesertLizard, Behavior::Moving) => {
                format!("A {} skitters across the hot sand.", name)
            }

            (Species::Scorpion, _) => format!("A {} lurks beneath a rock, pincers raised.", name),

            (Species::DesertFox, Behavior::Resting) => {
                format!("A {} with oversized ears rests in the shade.", name)
            }

            (Species::Hawk, Behavior::Hunting) => {
                format!("A {} circles overhead, riding thermal currents.", name)
            }
            (Species::Hawk, Behavior::Resting) => {
                format!("A {} perches on a dead branch, surveying its domain.", name)
            }

            (Species::SnowFox, Behavior::Hunting) => format!(
                "An {} blends almost invisibly with the snow, stalking prey.",
                name
            ),
            (Species::SnowFox, Behavior::Resting) => format!(
                "An {} curls into a perfect white ball against the snow.",
                name
            ),

            (Species::Owl, _) => format!(
                "A {} watches silently from a high branch, golden eyes unblinking.",
                name
            ),

            (Species::Wolf, Behavior::Moving) => format!(
                "A {} lopes through the snow, breath visible in the cold air.",
                name
            ),
            (Species::Wolf, Behavior::Alert) => {
                "In the distance, a wolf howls - a haunting, beautiful sound.".to_string()
            }

            (Species::Caribou, Behavior::Grazing) => {
                format!("A {} paws through snow to reach the lichen beneath.", name)
            }
            (Species::Caribou, Behavior::Moving) => {
                format!("A {} trudges through deep snow, antlers swaying.", name)
            }

            (Species::SnowHare, _) => {
                format!("A {} is barely visible against the white landscape.", name)
            }

            (Species::Duck, Behavior::Swimming) => {
                format!("A family of {}s glides across the still water.", name)
            }
            (Species::Duck, _) => format!("Several {}s bob gently on the lake's surface.", name),

            (Species::Fish, _) => {
                "A fish breaks the surface briefly, creating rippling circles.".to_string()
            }

            (Species::Heron, Behavior::Hunting) => format!(
                "A {} stands motionless in the shallows, waiting to strike.",
                name
            ),
            (Species::Heron, Behavior::Resting) => {
                format!("A {} preens its feathers on the shore.", name)
            }

            (Species::Frog, Behavior::Singing) => {
                "Frogs chorus in a symphony of croaks and chirps.".to_string()
            }
            (Species::Frog, _) => format!("A {} sits on a lily pad, throat pulsing.", name),

            (Species::Dragonfly, _) => format!(
                "A {} hovers over the water, wings catching the light like stained glass.",
                name
            ),

            (Species::Butterfly, _) => format!("A {} drifts lazily among the wildflowers.", name),
            (Species::Bee, _) => format!("A {} buzzes busily from flower to flower.", name),
            (Species::Pig, Behavior::Grazing) => {
                "A small pig snuffles through the grass, snout rooting gently in the soil.".to_string()
            }
            (Species::Pig, Behavior::Resting) => {
                "A small pig lies on its side in the clearing, sides rising and falling with slow breaths.".to_string()
            }
            (Species::Pig, Behavior::Moving) => {
                "A small pig trots along the path near the cabin, ears flicking at every sound.".to_string()
            }
            (Species::Pig, Behavior::Alert) => {
                "A small pig freezes for a moment, nose lifted as it tests the air.".to_string()
            }

            (_, Behavior::Sleeping) => format!("A {} sleeps peacefully.", name),
            (_, Behavior::Fleeing) => format!("A {} darts away, startled.", name),
            (_, behavior) => format!("A {} is {} nearby.", name, behavior.verb()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivitySchedule {
    Diurnal,     // Active during day
    Nocturnal,   // Active at night
    Crepuscular, // Active at dawn/dusk
}

impl ActivitySchedule {
    pub fn is_active(&self, time: TimeOfDay) -> bool {
        match self {
            ActivitySchedule::Diurnal => matches!(
                time,
                TimeOfDay::Morning | TimeOfDay::Noon | TimeOfDay::Afternoon
            ),
            ActivitySchedule::Nocturnal => matches!(
                time,
                TimeOfDay::Night | TimeOfDay::Midnight | TimeOfDay::Evening
            ),
            ActivitySchedule::Crepuscular => matches!(
                time,
                TimeOfDay::Dawn | TimeOfDay::Dusk | TimeOfDay::Morning | TimeOfDay::Evening
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Behavior {
    Sleeping,
    Resting,
    Grazing,
    Foraging,
    Hunting,
    Moving,
    Alert,
    Fleeing,
    Swimming,
    Singing,
    Basking,
}

impl Behavior {
    pub fn verb(&self) -> &'static str {
        match self {
            Behavior::Sleeping => "sleeping",
            Behavior::Resting => "resting",
            Behavior::Grazing => "grazing",
            Behavior::Foraging => "foraging",
            Behavior::Hunting => "hunting",
            Behavior::Moving => "moving",
            Behavior::Alert => "alert",
            Behavior::Fleeing => "fleeing",
            Behavior::Swimming => "swimming",
            Behavior::Singing => "singing",
            Behavior::Basking => "basking",
        }
    }

    pub fn random_for(species: &Species, time: TimeOfDay) -> Behavior {
        let schedule = species.activity_schedule();
        let mut rng = rand::thread_rng();

        if !schedule.is_active(time) {
            return if rng.gen_bool(0.8) {
                Behavior::Sleeping
            } else {
                Behavior::Resting
            };
        }

        let behaviors: Vec<Behavior> = match species {
            Species::Deer | Species::Caribou => vec![
                Behavior::Grazing,
                Behavior::Grazing,
                Behavior::Moving,
                Behavior::Alert,
                Behavior::Resting,
            ],
            Species::Rabbit | Species::SnowHare | Species::Squirrel => vec![
                Behavior::Foraging,
                Behavior::Foraging,
                Behavior::Moving,
                Behavior::Alert,
                Behavior::Resting,
            ],
            Species::Fox | Species::DesertFox | Species::SnowFox | Species::Wolf => vec![
                Behavior::Hunting,
                Behavior::Moving,
                Behavior::Resting,
                Behavior::Alert,
            ],
            Species::Songbird | Species::Frog => vec![
                Behavior::Singing,
                Behavior::Singing,
                Behavior::Moving,
                Behavior::Resting,
            ],
            Species::Duck => vec![
                Behavior::Swimming,
                Behavior::Swimming,
                Behavior::Foraging,
                Behavior::Resting,
            ],
            Species::DesertLizard => vec![
                Behavior::Basking,
                Behavior::Basking,
                Behavior::Moving,
                Behavior::Hunting,
            ],
            Species::Hawk | Species::Owl | Species::Heron => {
                vec![Behavior::Hunting, Behavior::Hunting, Behavior::Resting]
            }
            _ => vec![Behavior::Moving, Behavior::Resting, Behavior::Foraging],
        };

        behaviors[rng.gen_range(0..behaviors.len())]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wildlife {
    pub id: Uuid,
    pub species: Species,
    pub position: Position,
    pub behavior: Behavior,
    #[serde(default)]
    pub body: Body,
    #[serde(default = "Wildlife::default_alive")]
    pub alive: bool,
    #[serde(default)]
    pub tamed: bool,
}

impl Wildlife {
    pub fn new(species: Species, position: Position) -> Self {
        Self {
            id: Uuid::new_v4(),
            species,
            position,
            behavior: Behavior::Resting,
            body: Body::for_species(species),
            alive: true,
            tamed: false,
        }
    }

    fn default_alive() -> bool {
        true
    }

    pub fn update(&mut self, time: TimeOfDay, map: &WorldMap, weather: &RegionalWeather) {
        // Tamed companions mostly let the game state drive their movement.
        if self.tamed && matches!(self.species, Species::Dog | Species::Cat) {
            self.behavior = Behavior::Moving;
            return;
        }
        let mut rng = rand::thread_rng();
        let weather_here = weather.get_for_position(self.position.row, self.position.col);

        let severe = matches!(
            weather_here,
            Weather::Sandstorm | Weather::Blizzard | Weather::HeavyRain | Weather::HeavySnow
        );

        // Update behavior
        if rng.gen_bool(0.3) {
            // If off-schedule or severe weather, bias toward rest/sleep
            let schedule = self.species.activity_schedule();
            if !schedule.is_active(time) || severe {
                self.behavior = if rng.gen_bool(0.7) {
                    Behavior::Sleeping
                } else {
                    Behavior::Resting
                };
            } else {
                self.behavior = Behavior::random_for(&self.species, time);
            }
        }

        // Possibly move (reduced if movement limbs are badly injured)
        let movement_factor = self.body.movement_factor();
        let mut wants_to_move =
            self.behavior == Behavior::Moving || self.behavior == Behavior::Fleeing;
        if movement_factor < 0.5 {
            // Badly injured legs: often stay put
            if rng.gen_bool(0.7) {
                wants_to_move = false;
            }
        }
        if wants_to_move && !(severe && rng.gen_bool(0.7)) {
            let directions = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ];
            let dir = directions[rng.gen_range(0..4)];
            let new_pos = self.position.move_in_direction(dir);
            if new_pos.is_valid() {
                if let Some((r, c)) = new_pos.as_usize() {
                    if let Some(tile) = map.get_tile(r, c) {
                        if self.species.native_biomes().contains(&tile.biome) {
                            self.position = new_pos;
                        }
                    }
                }
            }
        }
    }

    pub fn describe(&self) -> String {
        self.species.describe_action(self.behavior)
    }
}

/// Spawn initial wildlife for the world
pub fn spawn_wildlife() -> Vec<Wildlife> {
    let mut wildlife = Vec::new();
    let mut rng = rand::thread_rng();

    let mut spawn_rect = |species: Species,
                          count: u8,
                          row_range: std::ops::Range<i32>,
                          col_range: std::ops::Range<i32>| {
        for _ in 0..count {
            let row = rng.gen_range(row_range.clone());
            let col = rng.gen_range(col_range.clone());
            wildlife.push(Wildlife::new(species, Position::new(row, col)));
        }
    };

    // Spring/Autumn forest wildlife (North band, rows <= -4)
    spawn_rect(Species::Deer, 3, -12..-4, -4..5);
    spawn_rect(Species::Rabbit, 4, -11..-3, -4..5);
    spawn_rect(Species::Squirrel, 3, -10..-3, -3..4);
    spawn_rect(Species::Songbird, 5, -10..-3, -4..5);

    // Desert wildlife (West, cols <= -8)
    spawn_rect(Species::DesertLizard, 3, -3..5, -14..-7);
    spawn_rect(Species::Scorpion, 1, -2..3, -14..-8);
    spawn_rect(Species::DesertFox, 1, -2..4, -12..-8);
    spawn_rect(Species::Hawk, 1, -1..4, -10..-7);

    // Winter wildlife (East, cols >= 7)
    spawn_rect(Species::SnowFox, 2, -4..5, 7..13);
    spawn_rect(Species::Owl, 1, -3..4, 9..13);
    spawn_rect(Species::Caribou, 1, -3..3, 8..12);

    // Lake wildlife (lake band rows -6..0 cols -4..5)
    spawn_rect(Species::Duck, 4, -6..0, -4..5);
    spawn_rect(Species::Fish, 3, -6..0, -4..5);
    spawn_rect(Species::Heron, 1, -5..-1, -3..4);
    spawn_rect(Species::Frog, 2, -5..0, -3..4);
    spawn_rect(Species::Dragonfly, 2, -5..0, -3..4);

    // Cabin-adjacent wildlife (Path / Clearing near cabin)
    spawn_rect(Species::Pig, 3, 0..4, -1..2);
    spawn_rect(Species::Dog, 1, 0..3, -2..2);
    spawn_rect(Species::Cat, 1, -1..2, -2..2);

    wildlife
}
