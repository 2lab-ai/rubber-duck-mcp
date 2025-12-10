use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::world::{Position, Direction, Biome, TimeOfDay};

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
        }
    }

    pub fn native_biomes(&self) -> Vec<Biome> {
        match self {
            Species::Deer | Species::Rabbit | Species::Squirrel
            | Species::Songbird | Species::Woodpecker | Species::Fox => {
                vec![Biome::SpringForest, Biome::MixedForest]
            }
            Species::DesertLizard | Species::Scorpion | Species::DesertFox
            | Species::Hawk | Species::Rattlesnake => {
                vec![Biome::Desert, Biome::Oasis]
            }
            Species::SnowFox | Species::Owl | Species::Wolf
            | Species::Caribou | Species::SnowHare => {
                vec![Biome::WinterForest]
            }
            Species::Duck | Species::Fish | Species::Heron
            | Species::Frog | Species::Dragonfly => {
                vec![Biome::Lake, Biome::Oasis]
            }
            Species::Butterfly | Species::Bee => {
                vec![Biome::SpringForest, Biome::MixedForest, Biome::Oasis]
            }
        }
    }

    pub fn activity_schedule(&self) -> ActivitySchedule {
        match self {
            Species::Owl | Species::Wolf | Species::Scorpion => ActivitySchedule::Nocturnal,
            Species::Deer | Species::Rabbit | Species::Fox => ActivitySchedule::Crepuscular,
            _ => ActivitySchedule::Diurnal,
        }
    }

    pub fn is_predator(&self) -> bool {
        matches!(self, Species::Fox | Species::DesertFox | Species::SnowFox
            | Species::Hawk | Species::Wolf | Species::Owl | Species::Rattlesnake
            | Species::Scorpion | Species::Heron)
    }

    /// Generate a description snippet for this animal doing an action
    pub fn describe_action(&self, behavior: Behavior) -> String {
        let name = self.name();
        match (self, behavior) {
            (Species::Deer, Behavior::Grazing) => format!("A {} grazes peacefully on tender grass.", name),
            (Species::Deer, Behavior::Alert) => format!("A {} stands frozen, ears twitching at some distant sound.", name),
            (Species::Deer, Behavior::Moving) => format!("A {} bounds gracefully between the trees.", name),

            (Species::Rabbit, Behavior::Foraging) => format!("A {} nibbles on clover, nose twitching constantly.", name),
            (Species::Rabbit, Behavior::Alert) => format!("A {} sits upright, scanning for danger.", name),

            (Species::Squirrel, Behavior::Foraging) => format!("A {} busily gathers acorns, stuffing its cheeks.", name),
            (Species::Squirrel, Behavior::Moving) => format!("A {} scampers up a tree trunk in spiraling leaps.", name),

            (Species::Songbird, Behavior::Singing) => format!("A {} trills a beautiful melody from the branches.", name),
            (Species::Songbird, Behavior::Moving) => format!("A {} flutters between branches.", name),

            (Species::Woodpecker, _) => format!("A {} drums rhythmically against a tree trunk.", name),

            (Species::Fox, Behavior::Hunting) => format!("A {} stalks through the underbrush, focused and silent.", name),
            (Species::Fox, Behavior::Resting) => format!("A {} curls up in a sunny patch, tail wrapped around its nose.", name),

            (Species::DesertLizard, Behavior::Basking) => format!("A {} basks on a sun-warmed rock.", name),
            (Species::DesertLizard, Behavior::Moving) => format!("A {} skitters across the hot sand.", name),

            (Species::Scorpion, _) => format!("A {} lurks beneath a rock, pincers raised.", name),

            (Species::DesertFox, Behavior::Resting) => format!("A {} with oversized ears rests in the shade.", name),

            (Species::Hawk, Behavior::Hunting) => format!("A {} circles overhead, riding thermal currents.", name),
            (Species::Hawk, Behavior::Resting) => format!("A {} perches on a dead branch, surveying its domain.", name),

            (Species::SnowFox, Behavior::Hunting) => format!("An {} blends almost invisibly with the snow, stalking prey.", name),
            (Species::SnowFox, Behavior::Resting) => format!("An {} curls into a perfect white ball against the snow.", name),

            (Species::Owl, _) => format!("A {} watches silently from a high branch, golden eyes unblinking.", name),

            (Species::Wolf, Behavior::Moving) => format!("A {} lopes through the snow, breath visible in the cold air.", name),
            (Species::Wolf, Behavior::Alert) => {
                "In the distance, a wolf howls - a haunting, beautiful sound.".to_string()
            }

            (Species::Caribou, Behavior::Grazing) => format!("A {} paws through snow to reach the lichen beneath.", name),
            (Species::Caribou, Behavior::Moving) => format!("A {} trudges through deep snow, antlers swaying.", name),

            (Species::SnowHare, _) => format!("A {} is barely visible against the white landscape.", name),

            (Species::Duck, Behavior::Swimming) => format!("A family of {}s glides across the still water.", name),
            (Species::Duck, _) => format!("Several {}s bob gently on the lake's surface.", name),

            (Species::Fish, _) => "A fish breaks the surface briefly, creating rippling circles.".to_string(),

            (Species::Heron, Behavior::Hunting) => format!("A {} stands motionless in the shallows, waiting to strike.", name),
            (Species::Heron, Behavior::Resting) => format!("A {} preens its feathers on the shore.", name),

            (Species::Frog, Behavior::Singing) => "Frogs chorus in a symphony of croaks and chirps.".to_string(),
            (Species::Frog, _) => format!("A {} sits on a lily pad, throat pulsing.", name),

            (Species::Dragonfly, _) => format!("A {} hovers over the water, wings catching the light like stained glass.", name),

            (Species::Butterfly, _) => format!("A {} drifts lazily among the wildflowers.", name),
            (Species::Bee, _) => format!("A {} buzzes busily from flower to flower.", name),

            (_, Behavior::Sleeping) => format!("A {} sleeps peacefully.", name),
            (_, Behavior::Fleeing) => format!("A {} darts away, startled.", name),
            (_, behavior) => format!("A {} is {} nearby.", name, behavior.verb()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivitySchedule {
    Diurnal,    // Active during day
    Nocturnal,  // Active at night
    Crepuscular, // Active at dawn/dusk
}

impl ActivitySchedule {
    pub fn is_active(&self, time: TimeOfDay) -> bool {
        match self {
            ActivitySchedule::Diurnal => matches!(time,
                TimeOfDay::Morning | TimeOfDay::Noon | TimeOfDay::Afternoon),
            ActivitySchedule::Nocturnal => matches!(time,
                TimeOfDay::Night | TimeOfDay::Midnight | TimeOfDay::Evening),
            ActivitySchedule::Crepuscular => matches!(time,
                TimeOfDay::Dawn | TimeOfDay::Dusk | TimeOfDay::Morning | TimeOfDay::Evening),
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
            return if rng.gen_bool(0.8) { Behavior::Sleeping } else { Behavior::Resting };
        }

        let behaviors: Vec<Behavior> = match species {
            Species::Deer | Species::Caribou => vec![
                Behavior::Grazing, Behavior::Grazing, Behavior::Moving,
                Behavior::Alert, Behavior::Resting,
            ],
            Species::Rabbit | Species::SnowHare | Species::Squirrel => vec![
                Behavior::Foraging, Behavior::Foraging, Behavior::Moving,
                Behavior::Alert, Behavior::Resting,
            ],
            Species::Fox | Species::DesertFox | Species::SnowFox | Species::Wolf => vec![
                Behavior::Hunting, Behavior::Moving, Behavior::Resting, Behavior::Alert,
            ],
            Species::Songbird | Species::Frog => vec![
                Behavior::Singing, Behavior::Singing, Behavior::Moving, Behavior::Resting,
            ],
            Species::Duck => vec![
                Behavior::Swimming, Behavior::Swimming, Behavior::Foraging, Behavior::Resting,
            ],
            Species::DesertLizard => vec![
                Behavior::Basking, Behavior::Basking, Behavior::Moving, Behavior::Hunting,
            ],
            Species::Hawk | Species::Owl | Species::Heron => vec![
                Behavior::Hunting, Behavior::Hunting, Behavior::Resting,
            ],
            _ => vec![
                Behavior::Moving, Behavior::Resting, Behavior::Foraging,
            ],
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
}

impl Wildlife {
    pub fn new(species: Species, position: Position) -> Self {
        Self {
            id: Uuid::new_v4(),
            species,
            position,
            behavior: Behavior::Resting,
        }
    }

    pub fn update(&mut self, time: TimeOfDay) {
        let mut rng = rand::thread_rng();

        // Update behavior
        if rng.gen_bool(0.3) {
            self.behavior = Behavior::random_for(&self.species, time);
        }

        // Possibly move
        if self.behavior == Behavior::Moving || self.behavior == Behavior::Fleeing {
            let directions = [Direction::North, Direction::South, Direction::East, Direction::West];
            let dir = directions[rng.gen_range(0..4)];
            let new_pos = self.position.move_in_direction(dir);
            if new_pos.is_valid() {
                self.position = new_pos;
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

    // Spring/Autumn forest wildlife (North area)
    for _ in 0..3 {
        let row = rng.gen_range(0..3);
        let col = rng.gen_range(3..8);
        wildlife.push(Wildlife::new(Species::Deer, Position::new(row, col)));
    }
    for _ in 0..4 {
        let row = rng.gen_range(0..4);
        let col = rng.gen_range(2..9);
        wildlife.push(Wildlife::new(Species::Rabbit, Position::new(row, col)));
    }
    for _ in 0..3 {
        let row = rng.gen_range(0..4);
        let col = rng.gen_range(3..8);
        wildlife.push(Wildlife::new(Species::Squirrel, Position::new(row, col)));
    }
    for _ in 0..5 {
        let row = rng.gen_range(0..5);
        let col = rng.gen_range(2..9);
        wildlife.push(Wildlife::new(Species::Songbird, Position::new(row, col)));
    }

    // Desert wildlife (West)
    for _ in 0..2 {
        let row = rng.gen_range(3..8);
        let col = rng.gen_range(0..2);
        wildlife.push(Wildlife::new(Species::DesertLizard, Position::new(row, col)));
    }
    wildlife.push(Wildlife::new(Species::DesertFox, Position::new(rng.gen_range(4..7), 0)));
    wildlife.push(Wildlife::new(Species::Hawk, Position::new(rng.gen_range(3..6), 1)));

    // Winter wildlife (East)
    for _ in 0..2 {
        let row = rng.gen_range(2..7);
        let col = rng.gen_range(9..11);
        wildlife.push(Wildlife::new(Species::SnowFox, Position::new(row, col)));
    }
    wildlife.push(Wildlife::new(Species::Owl, Position::new(rng.gen_range(3..6), 10)));
    wildlife.push(Wildlife::new(Species::Caribou, Position::new(rng.gen_range(2..5), rng.gen_range(9..11))));

    // Lake wildlife
    for _ in 0..4 {
        let row = rng.gen_range(1..5);
        let col = rng.gen_range(3..8);
        wildlife.push(Wildlife::new(Species::Duck, Position::new(row, col)));
    }
    for _ in 0..3 {
        let row = rng.gen_range(2..5);
        let col = rng.gen_range(2..9);
        wildlife.push(Wildlife::new(Species::Fish, Position::new(row, col)));
    }
    wildlife.push(Wildlife::new(Species::Heron, Position::new(rng.gen_range(1..4), rng.gen_range(2..8))));

    wildlife
}
