use super::{Biome, RegionalWeather, TimeOfDay, WorldMap, WorldTime};

/// Calculate the effective temperature at a position
pub fn calculate_temperature(
    map: &WorldMap,
    weather: &RegionalWeather,
    time: &WorldTime,
    pos: &super::map::Position,
    indoor: bool,
    fire_heat: f32,
) -> f32 {
    let (row, col) = pos.as_usize().unwrap_or((
        super::map::MAP_ORIGIN_ROW as usize,
        super::map::MAP_ORIGIN_COL as usize,
    ));
    let biome = map.get_biome_at(row, col).unwrap_or(Biome::MixedForest);
    let base_temp = biome.base_temperature();

    let time_mod = time.time_of_day().temperature_modifier();
    let weather_mod = weather
        .get_for_position(pos.row, pos.col)
        .temperature_modifier();

    let outdoor_temp = base_temp + time_mod + weather_mod;

    if indoor {
        // Indoor temperature is moderated
        let indoor_base = outdoor_temp * 0.5 + 10.0; // Insulated from extremes
        indoor_base + fire_heat
    } else {
        outdoor_temp
    }
}

/// Describe the sky based on time and weather
pub fn describe_sky(
    time: &WorldTime,
    weather: &RegionalWeather,
    row: i32,
    col: i32,
    biome: Biome,
) -> String {
    let tod = time.time_of_day();
    let current_weather = weather.get_for_position(row, col);

    let mut description = String::new();

    // Base sky description
    match (tod, current_weather) {
        (TimeOfDay::Dawn, _) => {
            description.push_str("The eastern horizon glows with the soft pink and gold of dawn. ");
        }
        (TimeOfDay::Morning, weather::Weather::Clear) => {
            description.push_str("The morning sky is a brilliant azure blue. ");
        }
        (TimeOfDay::Noon, weather::Weather::Clear) => {
            description.push_str("The sun blazes directly overhead in a cloudless sky. ");
        }
        (TimeOfDay::Afternoon, weather::Weather::Clear) => {
            description.push_str("The afternoon sun casts long, golden shadows. ");
        }
        (TimeOfDay::Dusk, _) => {
            description.push_str("The sky is painted in shades of orange, purple, and deep rose as the sun descends. ");
        }
        (TimeOfDay::Evening, weather::Weather::Clear) => {
            description.push_str("The first stars begin to appear in the darkening sky. ");
        }
        (TimeOfDay::Night, weather::Weather::Clear) => {
            description.push_str("A canopy of stars stretches across the velvet darkness. ");
        }
        (TimeOfDay::Midnight, weather::Weather::Clear) => {
            description.push_str("The world sleeps under a blanket of infinite stars. ");
        }
        (_, weather::Weather::Cloudy) => {
            description.push_str("Scattered clouds drift lazily across the sky. ");
        }
        (_, weather::Weather::Overcast) => {
            description.push_str("A thick layer of gray clouds covers the sky. ");
        }
        (_, weather::Weather::Fog) => {
            description.push_str("A thick fog obscures everything beyond a few meters. ");
        }
        (_, weather::Weather::LightRain) => {
            description.push_str("A gentle rain falls from gray skies. ");
        }
        (_, weather::Weather::HeavyRain) => {
            description.push_str("Heavy rain pours down, drumming against every surface. ");
        }
        (_, weather::Weather::LightSnow) => {
            description.push_str("Soft snowflakes drift down from the pale sky. ");
        }
        (_, weather::Weather::HeavySnow) => {
            description.push_str("Thick snow falls steadily, blanketing everything in white. ");
        }
        (_, weather::Weather::Blizzard) => {
            description
                .push_str("A fierce blizzard howls, reducing visibility to almost nothing. ");
        }
        (_, weather::Weather::Sandstorm) => {
            description.push_str("A wall of sand obscures the sky, stinging any exposed skin. ");
        }
        (_, weather::Weather::HeatWave) => match tod {
            TimeOfDay::Evening | TimeOfDay::Night | TimeOfDay::Midnight => {
                description.push_str(
                    "Even after dark, the air hangs heavy with trapped heat that refuses to fade. ",
                );
            }
            TimeOfDay::Dusk | TimeOfDay::Dawn => {
                description
                    .push_str("Heat haze warps the horizon, even as the sun sits low in the sky. ");
            }
            _ => {
                description.push_str("The air shimmers with intense heat under a merciless sun. ");
            }
        },
    }

    // Special features based on biome
    match biome {
        Biome::WinterForest
            if tod.aurora_visible()
                && matches!(
                    current_weather,
                    weather::Weather::Clear | weather::Weather::LightSnow
                ) =>
        {
            description.push_str("Ethereal ribbons of green and purple light dance across the sky - the aurora borealis. ");
        }
        Biome::SpringForest if tod.sunrise_visible() => {
            description.push_str("In the distance, the snow-capped peaks of the mountain range catch the early light, their summits glowing like beacons. ");
        }
        _ => {}
    }

    description
}

use super::weather;
