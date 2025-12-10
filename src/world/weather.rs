use rand::Rng;
use serde::{Deserialize, Serialize};
use super::map::Biome;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Cloudy,
    Overcast,
    LightRain,
    HeavyRain,
    Fog,
    Sandstorm,
    HeatWave,
    LightSnow,
    HeavySnow,
    Blizzard,
}

impl Weather {
    pub fn name(&self) -> &'static str {
        match self {
            Weather::Clear => "clear",
            Weather::Cloudy => "cloudy",
            Weather::Overcast => "overcast",
            Weather::LightRain => "light rain",
            Weather::HeavyRain => "heavy rain",
            Weather::Fog => "foggy",
            Weather::Sandstorm => "sandstorm",
            Weather::HeatWave => "heat wave",
            Weather::LightSnow => "light snow",
            Weather::HeavySnow => "heavy snow",
            Weather::Blizzard => "blizzard",
        }
    }

    pub fn visibility_modifier(&self) -> f32 {
        match self {
            Weather::Clear => 1.0,
            Weather::Cloudy => 0.9,
            Weather::Overcast => 0.7,
            Weather::LightRain => 0.6,
            Weather::HeavyRain => 0.3,
            Weather::Fog => 0.2,
            Weather::Sandstorm => 0.1,
            Weather::HeatWave => 0.8,
            Weather::LightSnow => 0.7,
            Weather::HeavySnow => 0.4,
            Weather::Blizzard => 0.1,
        }
    }

    pub fn temperature_modifier(&self) -> f32 {
        match self {
            Weather::Clear => 0.0,
            Weather::Cloudy => -2.0,
            Weather::Overcast => -4.0,
            Weather::LightRain => -5.0,
            Weather::HeavyRain => -7.0,
            Weather::Fog => -2.0,
            Weather::Sandstorm => 5.0,
            Weather::HeatWave => 10.0,
            Weather::LightSnow => -3.0,
            Weather::HeavySnow => -8.0,
            Weather::Blizzard => -15.0,
        }
    }

    pub fn possible_for_biome(biome: Biome) -> Vec<Weather> {
        match biome {
            Biome::Desert => vec![
                Weather::Clear, Weather::Clear, Weather::Clear, // Higher chance
                Weather::HeatWave, Weather::Sandstorm,
            ],
            Biome::Oasis => vec![
                Weather::Clear, Weather::Clear, Weather::Cloudy,
                Weather::HeatWave,
            ],
            Biome::SpringForest => vec![
                Weather::Clear, Weather::Cloudy, Weather::Overcast,
                Weather::LightRain, Weather::Fog,
            ],
            Biome::WinterForest => vec![
                Weather::Clear, Weather::Cloudy, Weather::Overcast,
                Weather::LightSnow, Weather::HeavySnow, Weather::Blizzard,
            ],
            Biome::Lake => vec![
                Weather::Clear, Weather::Cloudy, Weather::Fog,
            ],
            Biome::MixedForest | Biome::Path => vec![
                Weather::Clear, Weather::Cloudy, Weather::Overcast,
                Weather::LightRain, Weather::Fog,
            ],
        }
    }

    pub fn random_for_biome(biome: Biome) -> Weather {
        let possible = Self::possible_for_biome(biome);
        let mut rng = rand::thread_rng();
        possible[rng.gen_range(0..possible.len())]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalWeather {
    pub north: Weather,  // Spring/Autumn
    pub south: Weather,  // Mixed
    pub east: Weather,   // Winter
    pub west: Weather,   // Desert
}

impl RegionalWeather {
    pub fn new() -> Self {
        Self {
            north: Weather::random_for_biome(Biome::SpringForest),
            south: Weather::random_for_biome(Biome::MixedForest),
            east: Weather::random_for_biome(Biome::WinterForest),
            west: Weather::random_for_biome(Biome::Desert),
        }
    }

    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();

        // 20% chance to change weather each update
        if rng.gen_bool(0.2) {
            self.north = Weather::random_for_biome(Biome::SpringForest);
        }
        if rng.gen_bool(0.2) {
            self.south = Weather::random_for_biome(Biome::MixedForest);
        }
        if rng.gen_bool(0.2) {
            self.east = Weather::random_for_biome(Biome::WinterForest);
        }
        if rng.gen_bool(0.2) {
            self.west = Weather::random_for_biome(Biome::Desert);
        }
    }

    pub fn get_for_position(&self, row: i32, col: i32) -> Weather {
        // Calculate which direction dominates
        let center = 5.5f32;
        let row_diff = row as f32 - center;
        let col_diff = col as f32 - center;

        if row_diff.abs() > col_diff.abs() {
            if row_diff < 0.0 { self.north } else { self.south }
        } else {
            if col_diff < 0.0 { self.west } else { self.east }
        }
    }
}

impl Default for RegionalWeather {
    fn default() -> Self {
        Self::new()
    }
}
