use serde::{Deserialize, Serialize};
use crate::world::{Position, Direction};
use super::objects::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub woodcutting: u8,   // 1-100
    pub fire_making: u8,   // 1-100
    pub observation: u8,   // 1-100
    pub foraging: u8,      // 1-100
}

impl Skills {
    pub fn new() -> Self {
        Self {
            woodcutting: 10,
            fire_making: 10,
            observation: 10,
            foraging: 10,
        }
    }

    pub fn improve(&mut self, skill: &str, amount: u8) {
        let skill_ref = match skill {
            "woodcutting" => &mut self.woodcutting,
            "fire_making" => &mut self.fire_making,
            "observation" => &mut self.observation,
            "foraging" => &mut self.foraging,
            _ => return,
        };
        *skill_ref = (*skill_ref).saturating_add(amount).min(100);
    }

    pub fn get(&self, skill: &str) -> u8 {
        match skill {
            "woodcutting" => self.woodcutting,
            "fire_making" => self.fire_making,
            "observation" => self.observation,
            "foraging" => self.foraging,
            _ => 0,
        }
    }
}

impl Default for Skills {
    fn default() -> Self {
        Self::new()
    }
}

/// Interior room the player can be in
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Room {
    CabinMain,
    CabinTerrace,
    WoodShed,
}

impl Room {
    pub fn name(&self) -> &'static str {
        match self {
            Room::CabinMain => "cabin main room",
            Room::CabinTerrace => "cabin terrace",
            Room::WoodShed => "wood shed",
        }
    }

    pub fn is_indoor(&self) -> bool {
        matches!(self, Room::CabinMain | Room::WoodShed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item: Item,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub max_weight: f32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            max_weight: 50.0,
        }
    }

    pub fn current_weight(&self) -> f32 {
        self.slots.iter()
            .map(|slot| slot.item.weight() * slot.quantity as f32)
            .sum()
    }

    pub fn can_carry(&self, item: &Item, qty: u32) -> bool {
        self.current_weight() + item.weight() * qty as f32 <= self.max_weight
    }

    pub fn add(&mut self, item: Item, quantity: u32) -> bool {
        if !self.can_carry(&item, quantity) {
            return false;
        }

        // Check if we already have this item
        if let Some(slot) = self.slots.iter_mut().find(|s| s.item == item) {
            slot.quantity += quantity;
        } else {
            self.slots.push(InventorySlot { item, quantity });
        }
        true
    }

    pub fn remove(&mut self, item: &Item, quantity: u32) -> bool {
        if let Some(idx) = self.slots.iter().position(|s| &s.item == item) {
            if self.slots[idx].quantity >= quantity {
                self.slots[idx].quantity -= quantity;
                if self.slots[idx].quantity == 0 {
                    self.slots.remove(idx);
                }
                return true;
            }
        }
        false
    }

    pub fn has(&self, item: &Item, quantity: u32) -> bool {
        self.slots.iter()
            .find(|s| &s.item == item)
            .map(|s| s.quantity >= quantity)
            .unwrap_or(false)
    }

    pub fn count(&self, item: &Item) -> u32 {
        self.slots.iter()
            .find(|s| &s.item == item)
            .map(|s| s.quantity)
            .unwrap_or(0)
    }

    pub fn list(&self) -> Vec<(Item, u32)> {
        self.slots.iter()
            .map(|s| (s.item.clone(), s.quantity))
            .collect()
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    // Position
    pub position: Position,
    pub facing: Direction,
    pub room: Option<Room>,

    // Stats
    pub health: f32,      // 0-100
    pub warmth: f32,      // 0-100 (50 = comfortable)
    pub energy: f32,      // 0-100
    pub mood: f32,        // 0-100

    // Progression
    pub skills: Skills,
    pub inventory: Inventory,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Position::new(10, 5), // Start position
            facing: Direction::North,
            room: None,

            health: 100.0,
            warmth: 50.0,
            energy: 100.0,
            mood: 70.0,

            skills: Skills::new(),
            inventory: Inventory::new(),
        }
    }

    pub fn is_indoor(&self) -> bool {
        self.room.as_ref().map(|r| r.is_indoor()).unwrap_or(false)
    }

    pub fn move_to(&mut self, pos: Position) {
        self.position = pos;
    }

    pub fn face(&mut self, dir: Direction) {
        self.facing = dir;
    }

    pub fn enter_room(&mut self, room: Room) {
        self.room = Some(room);
    }

    pub fn exit_room(&mut self) {
        self.room = None;
    }

    pub fn modify_health(&mut self, delta: f32) {
        self.health = (self.health + delta).clamp(0.0, 100.0);
    }

    pub fn modify_warmth(&mut self, delta: f32) {
        self.warmth = (self.warmth + delta).clamp(0.0, 100.0);
    }

    pub fn modify_energy(&mut self, delta: f32) {
        self.energy = (self.energy + delta).clamp(0.0, 100.0);
    }

    pub fn modify_mood(&mut self, delta: f32) {
        self.mood = (self.mood + delta).clamp(0.0, 100.0);
    }

    /// Calculate comfort level based on warmth
    pub fn comfort_description(&self) -> &'static str {
        match self.warmth {
            w if w < 20.0 => "freezing",
            w if w < 35.0 => "cold",
            w if w < 45.0 => "slightly chilly",
            w if w < 55.0 => "comfortable",
            w if w < 65.0 => "slightly warm",
            w if w < 80.0 => "warm",
            _ => "overheating",
        }
    }

    pub fn mood_description(&self) -> &'static str {
        match self.mood {
            m if m < 20.0 => "miserable",
            m if m < 40.0 => "melancholy",
            m if m < 60.0 => "neutral",
            m if m < 80.0 => "content",
            _ => "joyful",
        }
    }

    pub fn energy_description(&self) -> &'static str {
        match self.energy {
            e if e < 20.0 => "exhausted",
            e if e < 40.0 => "tired",
            e if e < 60.0 => "slightly fatigued",
            e if e < 80.0 => "energetic",
            _ => "fully rested",
        }
    }

    pub fn status_summary(&self) -> String {
        format!(
            "You feel {} and {}. Your energy level is {}.",
            self.comfort_description(),
            self.mood_description(),
            self.energy_description()
        )
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
