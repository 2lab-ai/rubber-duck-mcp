use rand::seq::SliceRandom;
use crate::world::*;
use crate::entity::*;

pub struct DescriptionGenerator;

impl DescriptionGenerator {
    /// Generate a full description of the player's current location
    pub fn describe_location(
        player: &Player,
        map: &WorldMap,
        time: &WorldTime,
        weather: &RegionalWeather,
        wildlife: &[Wildlife],
        cabin: &Cabin,
        wood_shed: &WoodShed,
    ) -> String {
        // If in a room, describe that instead
        if let Some(room) = &player.room {
            return Self::describe_room(room, cabin, wood_shed, time, weather);
        }

        let (row, col) = match player.position.as_usize() {
            Some(p) => p,
            None => return "You seem to be nowhere.".to_string(),
        };

        let tile = match map.get_tile(row, col) {
            Some(t) => t,
            None => return "You're in an indescribable void.".to_string(),
        };

        let mut description = String::new();

        // Time and weather intro
        description.push_str(&Self::time_weather_intro(time, weather, row as i32, col as i32, tile.biome));

        // Main location description
        description.push_str("\n\n");
        description.push_str(&Self::tile_description(tile, row, col, player.facing, map));

        // Sky description
        description.push_str("\n\n");
        description.push_str(&describe_sky(time, weather, row as i32, col as i32, tile.biome));

        // Visible wildlife
        let nearby_wildlife: Vec<_> = wildlife.iter()
            .filter(|w| w.position.distance_to(&player.position) < 3.0)
            .collect();

        if !nearby_wildlife.is_empty() {
            description.push_str("\n\n");
            let mut rng = rand::thread_rng();
            // Pick up to 3 wildlife to describe
            let to_describe: Vec<_> = nearby_wildlife.choose_multiple(&mut rng, 3.min(nearby_wildlife.len())).collect();
            for w in to_describe {
                description.push_str(&w.describe());
                description.push(' ');
            }
        }

        // Exits
        description.push_str("\n\n");
        description.push_str(&Self::describe_exits(row, col, map, cabin.door_open));

        description
    }

    fn time_weather_intro(time: &WorldTime, weather: &RegionalWeather, row: i32, col: i32, biome: Biome) -> String {
        let tod = time.time_of_day();
        let current_weather = weather.get_for_position(row, col);

        let time_phrase = match tod {
            TimeOfDay::Dawn => "As dawn breaks",
            TimeOfDay::Morning => "In the bright morning light",
            TimeOfDay::Noon => "Under the midday sun",
            TimeOfDay::Afternoon => "In the lazy afternoon",
            TimeOfDay::Dusk => "As dusk settles",
            TimeOfDay::Evening => "In the soft evening darkness",
            TimeOfDay::Night => "Under the night sky",
            TimeOfDay::Midnight => "In the deep midnight hours",
        };

        let weather_phrase = match current_weather {
            Weather::Clear => "",
            Weather::Cloudy => ", clouds drift overhead",
            Weather::Overcast => ", gray clouds blanket the sky",
            Weather::LightRain => ", a gentle rain falls",
            Weather::HeavyRain => ", rain pours down around you",
            Weather::Fog => ", thick fog swirls around you",
            Weather::Sandstorm => ", sand whips through the air",
            Weather::HeatWave => ", the heat is almost unbearable",
            Weather::LightSnow => ", delicate snowflakes drift down",
            Weather::HeavySnow => ", heavy snow falls steadily",
            Weather::Blizzard => ", a fierce blizzard rages",
        };

        format!("{}{}, you find yourself in the {}.",
            time_phrase, weather_phrase, biome.name())
    }

    fn tile_description(tile: &Tile, row: usize, col: usize, facing: Direction, map: &WorldMap) -> String {
        match &tile.tile_type {
            TileType::Cabin => {
                "Before you stands a cozy wooden cabin. Its weathered planks speak of many seasons, \
                and a stone chimney rises from the shingled roof. A worn path leads to its wooden door."
                    .to_string()
            }
            TileType::Path => {
                let ahead = match facing {
                    Direction::North if row > 6 => "The path continues north toward a cabin.",
                    Direction::North => "The path opens into a forest clearing around a cabin.",
                    Direction::South if row < 10 => "The path stretches southward into the forest.",
                    Direction::South => "The path fades into the undergrowth at the world's edge.",
                    _ => "The worn path cuts through the forest.",
                };
                format!("You stand on a well-trodden dirt path through the forest. {}", ahead)
            }
            TileType::Forest(biome) => Self::forest_description(*biome),
            TileType::Lake => {
                "Crystal-clear water stretches before you, its surface like a mirror reflecting the sky. \
                Gentle ripples spread from somewhere near the center."
                    .to_string()
            }
            TileType::WoodShed => {
                "A small wooden shed stands here, its door slightly ajar.".to_string()
            }
        }
    }

    fn forest_description(biome: Biome) -> String {
        match biome {
            Biome::Desert => {
                "Waves of heat shimmer above the sand. Scattered cacti stand like silent sentinels, \
                their paddles dotted with tiny flowers. The sand shifts beneath your feet."
                    .to_string()
            }
            Biome::Oasis => {
                "Date palms sway gently around a pool of clear water. The air here is cooler, \
                refreshing after the desert heat. Colorful dragonflies dart above the water's edge."
                    .to_string()
            }
            Biome::SpringForest => {
                "Deciduous trees rise around you, their leaves a tapestry of greens. \
                Wildflowers dot the forest floor in patches of color. The air smells of fresh growth \
                and carries a hint of distant snow from the mountains to the north."
                    .to_string()
            }
            Biome::WinterForest => {
                "Snow blankets everything in pristine white. Evergreen trees stand proud beneath \
                their frozen burden, branches heavy with ice. Your breath fogs in the crisp, cold air. \
                The silence is profound, broken only by the occasional crack of freezing wood."
                    .to_string()
            }
            Biome::MixedForest => {
                "You stand in a mixed woodland of oak, maple, and pine. Shafts of light filter \
                through the canopy, dappling the leaf-covered ground. Birdsong echoes through the trees."
                    .to_string()
            }
            Biome::Path => {
                "The forest path stretches before you, worn smooth by countless footsteps.".to_string()
            }
            Biome::Lake => {
                "Reeds and lily pads line the water's edge. The lake reflects the sky like a mirror.".to_string()
            }
        }
    }

    fn describe_exits(row: usize, col: usize, map: &WorldMap, cabin_open: bool) -> String {
        let mut exits = Vec::new();

        // Check each direction
        let directions = [
            (Direction::North, -1, 0),
            (Direction::South, 1, 0),
            (Direction::East, 0, 1),
            (Direction::West, 0, -1),
        ];

        for (dir, dr, dc) in directions {
            let new_row = row as i32 + dr;
            let new_col = col as i32 + dc;

            if new_row >= 0 && new_col >= 0 {
                if let Some(tile) = map.get_tile(new_row as usize, new_col as usize) {
                    let exit_desc = match &tile.tile_type {
                        TileType::Lake => format!("{}: the lake waters", dir_str(dir)),
                        TileType::Cabin => {
                            if cabin_open {
                                format!("{}: the cabin (door open)", dir_str(dir))
                            } else {
                                format!("{}: the cabin (door closed)", dir_str(dir))
                            }
                        }
                        TileType::Path => format!("{}: the forest path", dir_str(dir)),
                        TileType::Forest(biome) => format!("{}: {}", dir_str(dir), biome.name()),
                        TileType::WoodShed => format!("{}: wood shed", dir_str(dir)),
                    };
                    exits.push(exit_desc);
                }
            }
        }

        if exits.is_empty() {
            "There are no obvious exits.".to_string()
        } else {
            format!("**Exits:** {}", exits.join(" | "))
        }
    }

    fn describe_room(room: &Room, cabin: &Cabin, wood_shed: &WoodShed, time: &WorldTime, weather: &RegionalWeather) -> String {
        match room {
            Room::CabinMain => Self::describe_cabin_main(cabin, time),
            Room::CabinTerrace => Self::describe_cabin_terrace(time, weather),
            Room::WoodShed => Self::describe_wood_shed(wood_shed),
        }
    }

    fn describe_cabin_main(cabin: &Cabin, time: &WorldTime) -> String {
        let tod = time.time_of_day();

        let light = match (tod, &cabin.fireplace.state) {
            (_, FireState::Roaring | FireState::Burning) =>
                "Warm firelight dances across the walls, casting flickering shadows.",
            (TimeOfDay::Morning | TimeOfDay::Noon | TimeOfDay::Afternoon, _) =>
                "Soft daylight filters through the windows.",
            (TimeOfDay::Dawn | TimeOfDay::Dusk, _) =>
                "Dim light seeps through the dusty windows.",
            _ => "The room is dark, save for faint moonlight through the windows.",
        };

        let fireplace_desc = cabin.fireplace.state.description();

        let items_on_ground: Vec<&str> = cabin.items.iter()
            .map(|i| i.name())
            .collect();

        let items_desc = if items_on_ground.is_empty() {
            String::new()
        } else {
            format!("\n\nScattered about you notice: {}.", items_on_ground.join(", "))
        };

        format!(
            "You are in the main room of the cabin. {}\n\n\
            A stone fireplace dominates one wall. {} \
            A wooden mantelpiece above it holds various curious items. \
            Worn but comfortable furniture fills the space - a sturdy table, \
            wooden chairs, and a faded rug that has seen better days.{}\n\n\
            **Exits:** North to terrace | West to wood shed | South to outside",
            light, fireplace_desc, items_desc
        )
    }

    fn describe_cabin_terrace(time: &WorldTime, weather: &RegionalWeather) -> String {
        let tod = time.time_of_day();

        let mut description = String::from(
            "You step onto the wooden terrace behind the cabin. The view opens up before you - \
            the lake stretches out, framed by the varied landscapes of this strange place.\n\n"
        );

        // Describe each direction
        description.push_str("**To the West:** ");
        let west_weather = weather.west;
        description.push_str(match west_weather {
            Weather::HeatWave | Weather::Clear if matches!(tod, TimeOfDay::Morning | TimeOfDay::Noon | TimeOfDay::Afternoon) =>
                "The desert shimmers in the heat. Palm trees dot the oasis edge, and you can just make out ripples in the cooler waters where desert meets lake.",
            Weather::Sandstorm =>
                "A wall of sand obscures the western horizon. The oasis palms bend in the fierce wind.",
            _ => "The desert landscape glows golden, its oasis a verdant jewel against the sand.",
        });

        description.push_str("\n\n**To the North:** ");
        let north_weather = weather.north;
        description.push_str(match (tod, north_weather) {
            (TimeOfDay::Dawn, _) =>
                "The distant snow-capped peaks catch the first light of dawn, glowing pink and gold. The lake's surface mirrors the spectacular colors. It's breathtaking.",
            (TimeOfDay::Morning, Weather::Clear) =>
                "The great mountains stand proud against a blue sky, their peaks eternally white. Mist rises from the lake.",
            (TimeOfDay::Dusk, _) =>
                "The mountains are silhouettes now, dark against a sky ablaze with sunset colors.",
            _ => "The temperate forest stretches north to where distant mountains scrape the sky.",
        });

        description.push_str("\n\n**To the East:** ");
        let east_weather = weather.east;
        description.push_str(match (tod, east_weather) {
            (_, Weather::Blizzard) =>
                "A fierce blizzard obscures the eastern shore. You can barely make out the shapes of snow-laden trees.",
            (TimeOfDay::Evening | TimeOfDay::Night | TimeOfDay::Midnight, Weather::Clear | Weather::LightSnow) =>
                "The snow-covered forest glitters under the aurora. Ribbons of green and purple light dance across the sky, reflected in the icy lake waters. Magnificent.",
            (_, Weather::HeavySnow) =>
                "Heavy snow falls on the eastern forest. Everything is white, peaceful, silent.",
            _ => "Snow blankets the eastern shore, evergreens standing like frozen sentinels.",
        });

        description.push_str("\n\n**Exits:** South back to cabin | West to wood shed");

        description
    }

    fn describe_wood_shed(wood_shed: &WoodShed) -> String {
        let axe_desc = if wood_shed.axe_on_floor {
            "A well-used axe lies on the earthen floor."
        } else {
            "The axe's usual spot on the floor is empty."
        };

        let log_desc = if wood_shed.logs > 0 {
            format!("A pile of unsplit logs leans against the wall - {} remain.", wood_shed.logs)
        } else {
            "The log pile is empty.".to_string()
        };

        let firewood_desc = if wood_shed.firewood > 0 {
            format!("Split firewood is stacked neatly nearby - {} pieces.", wood_shed.firewood)
        } else {
            "There's no split firewood.".to_string()
        };

        let block_desc = if wood_shed.chopping_block.has_log {
            "A log sits ready on the chopping block."
        } else {
            "The chopping block stands empty, its surface scarred from use."
        };

        format!(
            "You're in the small wood shed attached to the cabin. The air smells of sawdust and pine resin. \
            Dust motes drift in the light filtering through gaps in the wooden walls.\n\n\
            {} {} {} {}\n\n\
            **Exits:** East to cabin | North to terrace | South to outside",
            axe_desc, log_desc, firewood_desc, block_desc
        )
    }

    /// Describe what the player sees in a specific direction
    pub fn look_direction(
        dir: Direction,
        player: &Player,
        map: &WorldMap,
        time: &WorldTime,
        weather: &RegionalWeather,
        wildlife: &[Wildlife],
    ) -> String {
        // If in terrace, special viewing
        if matches!(player.room, Some(Room::CabinTerrace)) {
            return Self::terrace_look_direction(dir, time, weather);
        }

        let (row, col) = match player.position.as_usize() {
            Some(p) => p,
            None => return "You can't see in that direction.".to_string(),
        };

        let (dr, dc) = dir.delta();
        let look_row = row as i32 + dr;
        let look_col = col as i32 + dc;

        if !map.is_valid_position(look_row, look_col) {
            return match dir {
                Direction::North => "You gaze north, but there's nothing beyond the world's edge.".to_string(),
                Direction::South => "To the south, the forest seems to fade into mist.".to_string(),
                Direction::East => "The eastern horizon shows only endless expanse.".to_string(),
                Direction::West => "The western lands stretch beyond your sight.".to_string(),
                _ => "Nothing remarkable in that direction.".to_string(),
            };
        }

        let tile = map.get_tile(look_row as usize, look_col as usize).unwrap();

        let mut desc = format!("Looking {}: ", dir_str(dir));

        desc.push_str(&match &tile.tile_type {
            TileType::Lake => "The lake's surface glitters, stretching into the distance.".to_string(),
            TileType::Cabin => "The cabin stands waiting, its chimney silhouetted against the sky.".to_string(),
            TileType::Path => "A well-worn path winds through the trees.".to_string(),
            TileType::Forest(biome) => Self::distant_biome_description(*biome),
            TileType::WoodShed => "The small wood shed is visible.".to_string(),
        });

        // Check for wildlife in that direction
        let wildlife_there: Vec<_> = wildlife.iter()
            .filter(|w| {
                let wpos = w.position;
                let dir_to_w = (wpos.row - player.position.row, wpos.col - player.position.col);
                match dir {
                    Direction::North => dir_to_w.0 < 0 && dir_to_w.1.abs() <= 2,
                    Direction::South => dir_to_w.0 > 0 && dir_to_w.1.abs() <= 2,
                    Direction::East => dir_to_w.1 > 0 && dir_to_w.0.abs() <= 2,
                    Direction::West => dir_to_w.1 < 0 && dir_to_w.0.abs() <= 2,
                    _ => false,
                }
            })
            .take(2)
            .collect();

        if !wildlife_there.is_empty() {
            desc.push('\n');
            for w in wildlife_there {
                desc.push_str(&w.describe());
                desc.push(' ');
            }
        }

        desc
    }

    fn distant_biome_description(biome: Biome) -> String {
        match biome {
            Biome::Desert =>
                "Sand dunes ripple toward the horizon, shimmering with heat.".to_string(),
            Biome::Oasis =>
                "Palm fronds sway near a glimmering pool of water.".to_string(),
            Biome::SpringForest =>
                "A verdant forest of deciduous trees stretches into the distance.".to_string(),
            Biome::WinterForest =>
                "Snow-laden evergreens stand in silent rows, a winter wonderland.".to_string(),
            Biome::MixedForest =>
                "Mixed woodland extends as far as you can see.".to_string(),
            Biome::Lake =>
                "The lake's waters lap gently at the shore.".to_string(),
            Biome::Path =>
                "A path winds through the forest.".to_string(),
        }
    }

    fn terrace_look_direction(dir: Direction, time: &WorldTime, weather: &RegionalWeather) -> String {
        let tod = time.time_of_day();

        match dir {
            Direction::West => {
                let w = weather.west;
                format!("You gaze west over the lake toward the desert. {}",
                    match (tod, w) {
                        (_, Weather::Sandstorm) => "A massive sandstorm churns on the horizon, an awe-inspiring wall of dust and wind.",
                        (TimeOfDay::Noon, _) => "The desert blazes under the midday sun. Heat mirages dance above the dunes.",
                        (TimeOfDay::Dusk, _) => "The setting sun paints the desert in shades of gold and crimson.",
                        _ => "The oasis sparkles where desert meets lake, a precious jewel of green.",
                    }
                )
            }
            Direction::North => {
                format!("You look north across the lake toward the mountains. {}",
                    match tod {
                        TimeOfDay::Dawn => "The sunrise paints the distant peaks in brilliant pink and gold - a sight that could move anyone to tears.",
                        TimeOfDay::Dusk => "The mountains glow amber in the fading light, their eternal snows touched with rose.",
                        TimeOfDay::Night | TimeOfDay::Midnight => "The great peaks are silver shadows under the stars.",
                        _ => "The snow-capped mountains pierce the sky, impossibly distant and magnificent.",
                    }
                )
            }
            Direction::East => {
                let w = weather.east;
                format!("You turn your gaze eastward to the snowy forest. {}",
                    match (tod, w) {
                        (TimeOfDay::Evening | TimeOfDay::Night | TimeOfDay::Midnight, Weather::Clear | Weather::LightSnow) =>
                            "The aurora borealis dances in ethereal curtains of green and purple, reflecting off the frozen lake. It's magical.",
                        (_, Weather::Blizzard) => "A blizzard rages, the snowy forest nearly invisible in the swirling white.",
                        _ => "Everything is blanketed in pristine white snow. The silence is profound.",
                    }
                )
            }
            Direction::South => {
                "You look back at the cabin behind you. Warm and welcoming, smoke may or may not rise from its chimney.".to_string()
            }
            _ => "You can't look that direction from here.".to_string(),
        }
    }
}

fn dir_str(dir: Direction) -> &'static str {
    match dir {
        Direction::North => "North",
        Direction::South => "South",
        Direction::East => "East",
        Direction::West => "West",
        Direction::Up => "Up",
        Direction::Down => "Down",
    }
}
