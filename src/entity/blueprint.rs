use crate::entity::Item;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct BlueprintRecipe {
    target_item: Item,
    required: &'static [(Item, u32)],
    time_cost: u32, // minutes
}

const BLUEPRINT_RECIPES: &[BlueprintRecipe] = &[
    BlueprintRecipe {
        target_item: Item::StoneKnife,
        required: &[
            (Item::SharpStone, 1),
            (Item::Stick, 1),
            (Item::PlantFiber, 1),
        ],
        time_cost: 30,
    },
    BlueprintRecipe {
        target_item: Item::StoneAxe,
        required: &[(Item::SharpStone, 1), (Item::Stick, 1), (Item::Cordage, 1)],
        time_cost: 40,
    },
    BlueprintRecipe {
        target_item: Item::Campfire,
        required: &[(Item::Stone, 4), (Item::Kindling, 1), (Item::Log, 2)],
        time_cost: 20,
    },
    BlueprintRecipe {
        target_item: Item::Cordage,
        required: &[(Item::PlantFiber, 3)],
        time_cost: 10,
    },
    BlueprintRecipe {
        target_item: Item::FishingRod,
        required: &[(Item::Bamboo, 1), (Item::Stick, 1), (Item::Cordage, 1)],
        time_cost: 35,
    },
];

fn recipe_for(target: Item) -> Option<&'static BlueprintRecipe> {
    BLUEPRINT_RECIPES.iter().find(|r| r.target_item == target)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub target_item: Item,
    pub required: HashMap<Item, u32>,
    pub current: HashMap<Item, u32>,
    pub time_cost: u32, // Total time required in minutes
}

impl Blueprint {
    pub fn new(target: Item) -> Option<Self> {
        let recipe = recipe_for(target)?;
        let required: HashMap<Item, u32> = recipe.required.iter().copied().collect();

        Some(Blueprint {
            target_item: recipe.target_item,
            required,
            current: HashMap::new(),
            time_cost: recipe.time_cost,
        })
    }

    pub fn add_material(&mut self, item: Item) -> bool {
        if let Some(req_qty) = self.required.get(&item) {
            let cur_qty = self.current.entry(item).or_insert(0);
            if *cur_qty < *req_qty {
                *cur_qty += 1;
                return true;
            }
        }
        false
    }

    pub fn is_complete(&self) -> bool {
        for (item, qty) in &self.required {
            let cur = self.current.get(item).unwrap_or(&0);
            if cur < qty {
                return false;
            }
        }
        true
    }

    pub fn missing_materials(&self) -> Vec<(Item, u32)> {
        let mut missing = Vec::new();
        for (item, qty) in &self.required {
            let cur = self.current.get(item).unwrap_or(&0);
            if cur < qty {
                missing.push((*item, qty - cur));
            }
        }
        missing
    }

    pub fn progress_entries(&self) -> Vec<(Item, u32, u32)> {
        let mut entries: Vec<(Item, u32, u32)> = self
            .required
            .iter()
            .map(|(item, req)| (*item, *self.current.get(item).unwrap_or(&0), *req))
            .collect();
        entries.sort_by_key(|(item, _, _)| item.name());
        entries
    }

    pub fn progress_summary(&self) -> String {
        let parts: Vec<String> = self
            .progress_entries()
            .into_iter()
            .map(|(item, cur, req)| format!("{} {}/{}", item.name(), cur, req))
            .collect();
        parts.join(", ")
    }

    pub fn status_description(&self) -> String {
        if self.is_complete() {
            return format!(
                "Blueprint for {} is ready to assemble. Total build time: {} mins.",
                self.target_item.name(),
                self.time_cost
            );
        }

        format!(
            "Blueprint for {}. Progress: {}. Total build time: {} mins.",
            self.target_item.name(),
            self.progress_summary(),
            self.time_cost
        )
    }
}
