use serde::{Deserialize, Serialize};

pub const MAP_WIDTH: usize = 11;
pub const MAP_HEIGHT: usize = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Direction {
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Direction::North => (-1, 0),
            Direction::South => (1, 0),
            Direction::East => (0, 1),
            Direction::West => (0, -1),
            Direction::Up | Direction::Down => (0, 0),
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    pub fn from_str(s: &str) -> Option<Direction> {
        match s.to_lowercase().as_str() {
            "n" | "north" => Some(Direction::North),
            "s" | "south" => Some(Direction::South),
            "e" | "east" => Some(Direction::East),
            "w" | "west" => Some(Direction::West),
            "u" | "up" => Some(Direction::Up),
            "d" | "down" => Some(Direction::Down),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Biome {
    Desert,      // West - hot summer
    Oasis,       // West lake edge
    SpringForest, // North - mild spring/autumn
    WinterForest, // East - cold winter/snow
    Lake,        // Central lake
    MixedForest, // South and general forest
    Path,        // The path from start to cabin
}

impl Biome {
    pub fn base_temperature(&self) -> f32 {
        match self {
            Biome::Desert => 35.0,
            Biome::Oasis => 28.0,
            Biome::SpringForest => 18.0,
            Biome::WinterForest => -5.0,
            Biome::Lake => 15.0,
            Biome::MixedForest => 20.0,
            Biome::Path => 20.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Biome::Desert => "scorching desert",
            Biome::Oasis => "refreshing oasis",
            Biome::SpringForest => "temperate forest",
            Biome::WinterForest => "snowy forest",
            Biome::Lake => "tranquil lake",
            Biome::MixedForest => "mixed woodland",
            Biome::Path => "worn forest path",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Forest(Biome),
    Lake,
    Cabin,
    Path,
    WoodShed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub tile_type: TileType,
    pub biome: Biome,
    pub elevation: f32,
    pub walkable: bool,
}

impl Tile {
    pub fn new(tile_type: TileType, biome: Biome) -> Self {
        let walkable = !matches!(tile_type, TileType::Lake);
        Self {
            tile_type,
            biome,
            elevation: 0.0,
            walkable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMap {
    tiles: Vec<Vec<Tile>>,
}

impl WorldMap {
    pub fn new() -> Self {
        let mut tiles = Vec::with_capacity(MAP_HEIGHT);

        for row in 0..MAP_HEIGHT {
            let mut row_tiles = Vec::with_capacity(MAP_WIDTH);
            for col in 0..MAP_WIDTH {
                let tile = Self::generate_tile(row, col);
                row_tiles.push(tile);
            }
            tiles.push(row_tiles);
        }

        WorldMap { tiles }
    }

    fn generate_tile(row: usize, col: usize) -> Tile {
        // Determine biome based on position
        let biome = Self::determine_biome(row, col);
        let tile_type = Self::determine_tile_type(row, col, biome);

        Tile::new(tile_type, biome)
    }

    fn determine_biome(row: usize, col: usize) -> Biome {
        // Lake area: rows 1-5, cols 1-9
        let is_lake_area = row >= 1 && row <= 5 && col >= 1 && col <= 9;

        if is_lake_area {
            // Lake edges take on adjacent biome characteristics
            if col <= 2 {
                return Biome::Oasis;
            } else if col >= 8 {
                return Biome::Lake; // Winter side of lake
            }
            return Biome::Lake;
        }

        // West side - Desert
        if col <= 1 {
            return Biome::Desert;
        }

        // East side - Winter
        if col >= 9 {
            return Biome::WinterForest;
        }

        // North side (rows 0-2) - Spring/Autumn forest
        if row <= 2 {
            return Biome::SpringForest;
        }

        // Path and cabin area
        if col == 5 && row >= 6 {
            return Biome::Path;
        }

        // Default - mixed forest
        Biome::MixedForest
    }

    fn determine_tile_type(row: usize, col: usize, biome: Biome) -> TileType {
        // Lake tiles
        if row >= 1 && row <= 5 && col >= 1 && col <= 9 {
            // Not oasis edges
            if !(col <= 2 && row >= 4) && !(col >= 8 && row >= 4) {
                return TileType::Lake;
            }
        }

        // Cabin at (6, 5)
        if row == 6 && col == 5 {
            return TileType::Cabin;
        }

        // Path from start (10, 5) to cabin
        if col == 5 && row >= 7 && row <= 10 {
            return TileType::Path;
        }

        // Forest everywhere else
        TileType::Forest(biome)
    }

    pub fn get_tile(&self, row: usize, col: usize) -> Option<&Tile> {
        self.tiles.get(row).and_then(|r| r.get(col))
    }

    pub fn get_tile_mut(&mut self, row: usize, col: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(row).and_then(|r| r.get_mut(col))
    }

    pub fn is_valid_position(&self, row: i32, col: i32) -> bool {
        row >= 0 && col >= 0 &&
        (row as usize) < MAP_HEIGHT &&
        (col as usize) < MAP_WIDTH
    }

    pub fn is_walkable(&self, row: usize, col: usize) -> bool {
        self.get_tile(row, col)
            .map(|t| t.walkable)
            .unwrap_or(false)
    }

    pub fn get_biome_at(&self, row: usize, col: usize) -> Option<Biome> {
        self.get_tile(row, col).map(|t| t.biome)
    }

    /// Calculate which seasonal biome direction dominates at this position
    pub fn get_dominant_direction(&self, row: usize, col: usize) -> Direction {
        let center_row = 5.5;
        let center_col = 5.5;

        let row_diff = row as f32 - center_row;
        let col_diff = col as f32 - center_col;

        if row_diff.abs() > col_diff.abs() {
            if row_diff < 0.0 { Direction::North } else { Direction::South }
        } else {
            if col_diff < 0.0 { Direction::West } else { Direction::East }
        }
    }
}

impl Default for WorldMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub row: i32,
    pub col: i32,
}

impl Position {
    pub fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }

    pub fn move_in_direction(&self, dir: Direction) -> Position {
        let (dr, dc) = dir.delta();
        Position {
            row: self.row + dr,
            col: self.col + dc,
        }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let dr = (self.row - other.row) as f32;
        let dc = (self.col - other.col) as f32;
        (dr * dr + dc * dc).sqrt()
    }

    pub fn is_valid(&self) -> bool {
        self.row >= 0 && self.col >= 0 &&
        (self.row as usize) < MAP_HEIGHT &&
        (self.col as usize) < MAP_WIDTH
    }

    pub fn as_usize(&self) -> Option<(usize, usize)> {
        if self.is_valid() {
            Some((self.row as usize, self.col as usize))
        } else {
            None
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}
