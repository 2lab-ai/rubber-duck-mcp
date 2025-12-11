use crate::entity::{Item, LocationItems};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const MAP_EXTENT: i32 = 50; // world coords span -50..50
pub const MAP_WIDTH: usize = (MAP_EXTENT as usize * 2) + 1;
pub const MAP_HEIGHT: usize = (MAP_EXTENT as usize * 2) + 1;
pub const MAP_ORIGIN_ROW: i32 = MAP_EXTENT;
pub const MAP_ORIGIN_COL: i32 = MAP_EXTENT;

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
    Desert,       // West - hot summer
    Oasis,        // West lake edge
    SpringForest, // North - mild spring/autumn
    WinterForest, // East - cold winter/snow
    Lake,         // Central lake
    MixedForest,  // South and general forest
    Path,         // The path from start to cabin
    BambooGrove,  // Lakeside bamboo
    Clearing,     // Cabin clearing
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
            Biome::BambooGrove => 22.0,
            Biome::Clearing => 20.0,
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
            Biome::BambooGrove => "bamboo grove",
            Biome::Clearing => "clearing",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Forest(Biome),
    Lake,
    Path,
    Clearing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub tile_type: TileType,
    pub biome: Biome,
    pub elevation: f32,
    pub walkable: bool,
    pub items: LocationItems,
}

impl Tile {
    pub fn new(tile_type: TileType, biome: Biome) -> Self {
        let walkable = !matches!(tile_type, TileType::Lake);
        let mut items = LocationItems::new();

        // Spawn basic resources (Stones) everywhere except deep lake
        if !matches!(tile_type, TileType::Lake) {
            let mut rng = rand::thread_rng();
            let stone_count = rng.gen_range(3..=10);
            items.add(Item::Stone, stone_count);
        }

        Self {
            tile_type,
            biome,
            elevation: 0.0,
            walkable,
            items,
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
        // Convert grid index to world coordinates (origin at cabin)
        let world_row = row as i32 - MAP_ORIGIN_ROW;
        let world_col = col as i32 - MAP_ORIGIN_COL;

        // Determine biome based on position
        let biome = Self::determine_biome(world_row, world_col);
        let tile_type = Self::determine_tile_type(world_row, world_col, biome);

        let mut tile = Tile::new(tile_type, biome);
        if world_row.abs() == MAP_EXTENT || world_col.abs() == MAP_EXTENT {
            tile.walkable = false; // Impassable border
        }
        tile
    }

    fn determine_biome(world_row: i32, world_col: i32) -> Biome {
        // Lake core: rows -5..-1, cols -4..4 (north of cabin)
        let is_lake_area = world_row >= -5 && world_row <= -1 && world_col >= -4 && world_col <= 4;
        if is_lake_area {
            if world_col <= -3 {
                return Biome::Oasis;
            }
            if world_col >= 3 {
                return Biome::Lake;
            }
            return Biome::Lake;
        }

        // Cabin clearing at (0,0) and adjacent (-1,-1)
        if (world_row == 0 && world_col == 0) || (world_row == -1 && world_col == -1) {
            return Biome::Clearing;
        }

        // Bamboo grove near lake south-west edge
        if world_row >= 0 && world_row <= 1 && world_col >= -3 && world_col <= -1 {
            return Biome::BambooGrove;
        }

        // West side - Desert band
        if world_col <= -5 {
            return Biome::Desert;
        }

        // East side - Winter
        if world_col >= 5 {
            return Biome::WinterForest;
        }

        // North band (rows <= -4) - Spring/Autumn forest
        if world_row <= -4 {
            return Biome::SpringForest;
        }

        // Path south to start (col 0, rows 1..5)
        if world_col == 0 && world_row >= 1 && world_row <= 5 {
            return Biome::Path;
        }

        // Default south/mid - mixed forest
        Biome::MixedForest
    }

    fn determine_tile_type(world_row: i32, world_col: i32, biome: Biome) -> TileType {
        // Lake tiles
        if world_row >= -5 && world_row <= -1 && world_col >= -4 && world_col <= 4 {
            return TileType::Lake;
        }

        // Clearings for placed structures
        if matches!(biome, Biome::Clearing) {
            return TileType::Clearing;
        }

        // Path from start (row 5, col 0) to cabin clearing
        if world_col == 0 && world_row >= 1 && world_row <= 5 {
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
        let gr = row + MAP_ORIGIN_ROW;
        let gc = col + MAP_ORIGIN_COL;
        gr >= 0 && gc >= 0 && (gr as usize) < MAP_HEIGHT && (gc as usize) < MAP_WIDTH
    }

    pub fn is_walkable(&self, row: usize, col: usize) -> bool {
        self.get_tile(row, col).map(|t| t.walkable).unwrap_or(false)
    }

    pub fn get_biome_at(&self, row: usize, col: usize) -> Option<Biome> {
        self.get_tile(row, col).map(|t| t.biome)
    }

    /// Calculate which seasonal biome direction dominates at this position
    pub fn get_dominant_direction(&self, row: usize, col: usize) -> Direction {
        let row_diff = row as i32 - MAP_ORIGIN_ROW;
        let col_diff = col as i32 - MAP_ORIGIN_COL;

        if row_diff.abs() > col_diff.abs() {
            if row_diff < 0 {
                Direction::North
            } else {
                Direction::South
            }
        } else {
            if col_diff < 0 {
                Direction::West
            } else {
                Direction::East
            }
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
        let gr = self.row + MAP_ORIGIN_ROW;
        let gc = self.col + MAP_ORIGIN_COL;
        gr >= 0 && gc >= 0 && (gr as usize) < MAP_HEIGHT && (gc as usize) < MAP_WIDTH
    }

    pub fn as_usize(&self) -> Option<(usize, usize)> {
        if self.is_valid() {
            let gr = (self.row + MAP_ORIGIN_ROW) as usize;
            let gc = (self.col + MAP_ORIGIN_COL) as usize;
            Some((gr, gc))
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
