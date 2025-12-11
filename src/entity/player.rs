use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::world::{Position, Direction};
use super::objects::Item;
use super::blueprint::Blueprint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProgress {
    pub level: u8,
    pub xp: u32,
}

impl SkillProgress {
    pub fn new(level: u8) -> Self {
        Self { level, xp: 0 }
    }
}

const SKILL_IDS: &[&str] = &[
    "woodcutting",
    "fire_making",
    "observation",
    "foraging",
    "stonemasonry",
    "survival",
    "tailoring",
    "cooking",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub woodcutting: u8,   // 1-100
    pub fire_making: u8,   // 1-100
    pub observation: u8,   // 1-100
    pub foraging: u8,      // 1-100
    #[serde(default)]
    pub stonemasonry: u8,  // 1-100
    #[serde(default)]
    pub survival: u8,      // 1-100
    #[serde(default)]
    pub tailoring: u8,     // 1-100
    #[serde(default)]
    pub cooking: u8,       // 1-100
    #[serde(default)]
    pub progress: HashMap<String, SkillProgress>,
}

impl Skills {
    pub fn new() -> Self {
        let mut progress = HashMap::new();
        for id in SKILL_IDS {
            progress.insert((*id).to_string(), SkillProgress::new(10));
        }
        Self {
            woodcutting: 10,
            fire_making: 10,
            observation: 10,
            foraging: 10,
            stonemasonry: 10,
            survival: 10,
            tailoring: 10,
            cooking: 10,
            progress,
        }
    }

    fn xp_to_next(level: u8) -> u32 {
        10 + level as u32 * 5
    }

    fn level_slot(&mut self, skill: &str) -> Option<&mut u8> {
        match skill {
            "woodcutting" => Some(&mut self.woodcutting),
            "fire_making" => Some(&mut self.fire_making),
            "observation" => Some(&mut self.observation),
            "foraging" => Some(&mut self.foraging),
            "stonemasonry" => Some(&mut self.stonemasonry),
            "survival" => Some(&mut self.survival),
            "tailoring" => Some(&mut self.tailoring),
            "cooking" => Some(&mut self.cooking),
            _ => None,
        }
    }

    fn field_level(&self, skill: &str) -> u8 {
        match skill {
            "woodcutting" => self.woodcutting,
            "fire_making" => self.fire_making,
            "observation" => self.observation,
            "foraging" => self.foraging,
            "stonemasonry" => self.stonemasonry,
            "survival" => self.survival,
            "tailoring" => self.tailoring,
            "cooking" => self.cooking,
            _ => 0,
        }
    }

    fn progress_entry(&mut self, skill: &str) -> Option<&mut SkillProgress> {
        if !SKILL_IDS.contains(&skill) {
            return None;
        }
        let seed_level = self.field_level(skill);
        Some(self.progress.entry(skill.to_string()).or_insert_with(|| SkillProgress::new(seed_level)))
    }

    pub fn improve(&mut self, skill: &str, amount: u8) {
        let Some(level_after) = ({
            let Some(progress) = self.progress_entry(skill) else { return; };
            progress.xp = progress.xp.saturating_add(amount as u32);
            while progress.xp >= Self::xp_to_next(progress.level) && progress.level < 100 {
                progress.xp -= Self::xp_to_next(progress.level);
                progress.level = progress.level.saturating_add(1);
            }
            Some(progress.level)
        }) else {
            return;
        };
        if let Some(level_ref) = self.level_slot(skill) {
            *level_ref = level_after;
        }
    }

    pub fn get(&self, skill: &str) -> u8 {
        self.progress
            .get(skill)
            .map(|p| p.level)
            .unwrap_or_else(|| self.field_level(skill))
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
    #[serde(default = "Player::default_fullness")]
    pub fullness: f32,    // 0-100 (hunger)
    #[serde(default = "Player::default_hydration")]
    pub hydration: f32,   // 0-100 (thirst)

    // Progression
    pub skills: Skills,
    pub inventory: Inventory,
    
    // Crafting
    #[serde(default)]
    pub active_project: Option<Blueprint>,
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
            fullness: Self::default_fullness(),
            hydration: Self::default_hydration(),

            skills: Skills::new(),
            inventory: Inventory::new(),
            active_project: None,
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

    pub fn modify_fullness(&mut self, delta: f32) {
        self.fullness = (self.fullness + delta).clamp(0.0, 100.0);
    }

    pub fn modify_hydration(&mut self, delta: f32) {
        self.hydration = (self.hydration + delta).clamp(0.0, 100.0);
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

    pub fn fullness_description(&self) -> &'static str {
        match self.fullness {
            f if f < 20.0 => "very hungry",
            f if f < 40.0 => "a bit hungry",
            f if f < 60.0 => "satisfied",
            f if f < 80.0 => "comfortable and full",
            _ => "stuffed",
        }
    }

    pub fn hydration_description(&self) -> &'static str {
        match self.hydration {
            h if h < 20.0 => "parched",
            h if h < 40.0 => "thirsty",
            h if h < 60.0 => "quenched",
            h if h < 80.0 => "well hydrated",
            _ => "brimming",
        }
    }

    pub fn status_summary(&self) -> String {
        format!(
            "You feel {} and {}. Your energy level is {}. You are {} and {}.",
            self.comfort_description(),
            self.mood_description(),
            self.energy_description(),
            self.fullness_description(),
            self.hydration_description(),
        )
    }

    fn default_fullness() -> f32 { 70.0 }
    fn default_hydration() -> f32 { 70.0 }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
