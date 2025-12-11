use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::entity::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub target_item: Item,
    pub required: HashMap<Item, u32>,
    pub current: HashMap<Item, u32>,
    pub time_cost: u32, // Total time required in minutes
}

impl Blueprint {
    pub fn new(target: Item) -> Option<Self> {
        match target {
            Item::StoneKnife => Some(Blueprint {
                target_item: Item::StoneKnife,
                required: HashMap::from([
                    (Item::SharpStone, 1),
                    (Item::Stick, 1),
                    (Item::PlantFiber, 1), // Binding
                ]),
                current: HashMap::new(),
                time_cost: 30,
            }),
            Item::StoneAxe => Some(Blueprint {
                target_item: Item::StoneAxe,
                required: HashMap::from([
                    (Item::SharpStone, 1),
                    (Item::Stick, 1),
                    (Item::Cordage, 1),
                ]),
                current: HashMap::new(),
                time_cost: 40,
            }),
            Item::Campfire => Some(Blueprint {
                target_item: Item::Campfire,
                required: HashMap::from([
                    (Item::Stone, 4),
                    (Item::Kindling, 1),
                    (Item::Log, 2),
                ]),
                current: HashMap::new(),
                time_cost: 20,
            }),
            Item::Cordage => Some(Blueprint {
                target_item: Item::Cordage,
                required: HashMap::from([
                    (Item::PlantFiber, 3),
                ]),
                current: HashMap::new(),
                time_cost: 10,
            }),
            _ => None,
        }
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
    
    pub fn status_description(&self) -> String {
        let missing = self.missing_materials();
        if missing.is_empty() {
            return format!("Blueprint for {} (Ready to assemble)", self.target_item.name());
        }
        
        let req_list: Vec<String> = missing.iter()
            .map(|(item, qty)| format!("{} x{}", item.name(), qty))
            .collect();
            
        format!("Blueprint for {}. Needs: {}", self.target_item.name(), req_list.join(", "))
    }
}
