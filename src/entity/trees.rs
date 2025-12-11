use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::world::Position;
use crate::entity::objects::Item;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TreeType {
    Pine,
    Birch,
    Apple,
    Bamboo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub position: Position,
    pub kind: TreeType,
    pub hits_done: u8,
    pub hits_required: u8,
    pub felled: bool,
    #[serde(default)]
    pub fruit_count: u8,
    #[serde(default)]
    pub fruit_max: u8,
}

impl Tree {
    pub fn new(position: Position, kind: TreeType) -> Self {
        let mut tree = Self {
            position,
            kind,
            hits_done: 0,
            hits_required: 5,
            felled: false,
            fruit_count: 0,
            fruit_max: 0,
        };
        tree.apply_kind_defaults();
        tree
    }

    pub fn with_random_fruit(position: Position, kind: TreeType, rng: &mut impl Rng) -> Self {
        let mut tree = Self::new(position, kind);
        if tree.fruit_max > 0 {
            let start = rng.gen_range(0..=tree.fruit_max.min(3));
            tree.fruit_count = start;
        }
        tree
    }

    pub fn description(&self) -> &'static str {
        match self.kind {
            TreeType::Pine => "A tall pine stands here, sap-heavy and straight.",
            TreeType::Birch => "A slender birch with pale bark and delicate branches.",
            TreeType::Apple => "A hardy apple tree, its branches often heavy with fruit.",
            TreeType::Bamboo => "A cluster of bamboo stalks sways softly in the breeze.",
        }
    }

    pub fn fruit_item(&self) -> Option<Item> {
        match self.kind {
            TreeType::Apple => Some(Item::Apple),
            _ => None,
        }
    }

    pub fn has_fruit(&self) -> bool {
        self.fruit_count > 0 && self.fruit_item().is_some()
    }

    pub fn take_fruit(&mut self, amount: u8) -> u8 {
        let taken = amount.min(self.fruit_count);
        self.fruit_count -= taken;
        taken
    }

    pub fn take_all_fruit(&mut self) -> u8 {
        let all = self.fruit_count;
        self.fruit_count = 0;
        all
    }

    pub fn tick_growth(&mut self, rng: &mut impl Rng) {
        if self.felled || self.fruit_max == 0 || self.fruit_count >= self.fruit_max {
            return;
        }
        if rng.gen_bool(0.18) {
            self.fruit_count = (self.fruit_count + 1).min(self.fruit_max);
        }
    }

    pub fn apply_kind_defaults(&mut self) {
        self.fruit_max = match self.kind {
            TreeType::Apple => 6,
            _ => 0,
        };
        if self.hits_required == 0 {
            self.hits_required = 5;
        }
        if let TreeType::Bamboo = self.kind {
            self.hits_required = 3;
        }
        if self.fruit_count > self.fruit_max {
            self.fruit_count = self.fruit_max;
        }
    }

    pub fn progress_text(&self) -> String {
        format!("Chopping progress: {}/{}", self.hits_done, self.hits_required)
    }

    pub fn default_trees() -> Vec<Self> {
        Vec::new()
    }
}
