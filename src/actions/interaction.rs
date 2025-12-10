use crate::entity::{Player, Room, Item, Cabin, WoodShed};

pub enum InteractionResult {
    Success(String),
    Failure(String),
    ItemObtained(Item, String),
    ItemLost(Item, String),
}

/// Open something (doors, containers)
pub fn try_open(target: &str, player: &Player, cabin: &mut Cabin) -> InteractionResult {
    let normalized = target.to_lowercase();

    // Cabin door
    if normalized.contains("door") || normalized.contains("cabin") {
        if cabin.door_open {
            return InteractionResult::Failure(
                "The door is already open.".to_string()
            );
        }

        // Check if player is near cabin
        let cabin_pos = crate::world::Position::new(6, 5);
        let near_cabin = player.position.distance_to(&cabin_pos) <= 1.5 ||
            matches!(player.room, Some(Room::CabinMain));

        if !near_cabin {
            return InteractionResult::Failure(
                "You're too far from the cabin door.".to_string()
            );
        }

        cabin.door_open = true;
        InteractionResult::Success(
            "You grasp the worn iron handle and pull. The door swings open with a soft creak, revealing the dim interior of the cabin.".to_string()
        )
    } else {
        InteractionResult::Failure(format!("You don't see a '{}' to open.", target))
    }
}

/// Close something
pub fn try_close(target: &str, player: &Player, cabin: &mut Cabin) -> InteractionResult {
    let normalized = target.to_lowercase();

    if normalized.contains("door") || normalized.contains("cabin") {
        if !cabin.door_open {
            return InteractionResult::Failure(
                "The door is already closed.".to_string()
            );
        }

        let cabin_pos = crate::world::Position::new(6, 5);
        let near_cabin = player.position.distance_to(&cabin_pos) <= 1.5 ||
            matches!(player.room, Some(Room::CabinMain));

        if !near_cabin {
            return InteractionResult::Failure(
                "You're too far from the cabin door.".to_string()
            );
        }

        cabin.door_open = false;
        InteractionResult::Success(
            "You push the door closed. It latches with a satisfying click.".to_string()
        )
    } else {
        InteractionResult::Failure(format!("You don't see a '{}' to close.", target))
    }
}

/// Take an item from the environment
pub fn try_take(
    item_name: &str,
    player: &mut Player,
    cabin: &mut Cabin,
    wood_shed: &mut WoodShed,
) -> InteractionResult {
    let item = match Item::from_str(item_name) {
        Some(i) => i,
        None => return InteractionResult::Failure(
            format!("You don't know what '{}' is.", item_name)
        ),
    };

    // Check based on location
    match &player.room {
        Some(Room::CabinMain) => {
            // Can take items from cabin
            if cabin.take_item(&item) {
                if player.inventory.add(item.clone(), 1) {
                    return InteractionResult::ItemObtained(item.clone(),
                        format!("You pick up the {}.", item.name())
                    );
                } else {
                    cabin.add_item(item.clone());
                    return InteractionResult::Failure(
                        "Your inventory is too heavy to carry that.".to_string()
                    );
                }
            }
            // Check matchbox on mantle
            if item == Item::Matchbox && cabin.has_item(&Item::Matchbox) {
                if cabin.take_item(&Item::Matchbox) {
                    if player.inventory.add(Item::Matchbox, 1) {
                        return InteractionResult::ItemObtained(Item::Matchbox,
                            "You take the matchbox from the mantelpiece.".to_string()
                        );
                    }
                }
            }
        }
        Some(Room::WoodShed) => {
            // Axe
            if item == Item::Axe && wood_shed.axe_on_floor {
                wood_shed.axe_on_floor = false;
                if player.inventory.add(Item::Axe, 1) {
                    return InteractionResult::ItemObtained(Item::Axe,
                        "You pick up the heavy axe, feeling its weight in your hands.".to_string()
                    );
                } else {
                    wood_shed.axe_on_floor = true;
                    return InteractionResult::Failure(
                        "The axe is too heavy to add to your load.".to_string()
                    );
                }
            }
            // Log
            if item == Item::Log && wood_shed.logs > 0 {
                if player.inventory.add(Item::Log, 1) {
                    wood_shed.logs -= 1;
                    return InteractionResult::ItemObtained(Item::Log,
                        format!("You heft a heavy log from the pile. {} remain.", wood_shed.logs)
                    );
                } else {
                    return InteractionResult::Failure(
                        "You're carrying too much to take another log.".to_string()
                    );
                }
            }
            // Firewood
            if item == Item::Firewood && wood_shed.firewood > 0 {
                if player.inventory.add(Item::Firewood, 1) {
                    wood_shed.firewood -= 1;
                    return InteractionResult::ItemObtained(Item::Firewood,
                        "You gather a piece of split firewood.".to_string()
                    );
                }
            }
        }
        _ => {}
    }

    InteractionResult::Failure(format!("You don't see any {} here that you can take.", item_name))
}

/// Drop an item from inventory
pub fn try_drop(
    item_name: &str,
    player: &mut Player,
    cabin: &mut Cabin,
    wood_shed: &mut WoodShed,
) -> InteractionResult {
    let item = match Item::from_str(item_name) {
        Some(i) => i,
        None => return InteractionResult::Failure(
            format!("You don't know what '{}' is.", item_name)
        ),
    };

    if !player.inventory.has(&item, 1) {
        return InteractionResult::Failure(
            format!("You don't have any {}.", item.name())
        );
    }

    player.inventory.remove(&item, 1);

    // Place based on location
    match &player.room {
        Some(Room::CabinMain) => {
            cabin.add_item(item.clone());
        }
        Some(Room::WoodShed) => {
            match &item {
                Item::Axe => wood_shed.axe_on_floor = true,
                Item::Log => wood_shed.logs += 1,
                Item::Firewood => wood_shed.firewood += 1,
                _ => {} // Other items just disappear into the shed
            }
        }
        _ => {} // Outside, items just... go away for now
    }

    InteractionResult::ItemLost(item.clone(),
        format!("You set down the {}.", item.name())
    )
}

/// Examine something in detail
pub fn examine(target: &str, player: &Player, cabin: &Cabin, wood_shed: &WoodShed) -> String {
    let normalized = target.to_lowercase();

    // Check inventory first
    for (item, _) in player.inventory.list() {
        if item.name().to_lowercase().contains(&normalized) {
            return item.description().to_string();
        }
    }

    // Check location-specific things
    match &player.room {
        Some(Room::CabinMain) => {
            if normalized.contains("fire") || normalized.contains("hearth") {
                return cabin.fireplace.state.description().to_string();
            }
            if normalized.contains("mantle") || normalized.contains("mantelpiece") {
                let has_matches = cabin.has_item(&Item::Matchbox);
                if has_matches {
                    return "The wooden mantelpiece above the fireplace holds a small matchbox and a few curious trinkets.".to_string();
                } else {
                    return "The wooden mantelpiece is mostly bare, marked with rings from countless cups.".to_string();
                }
            }
            // Check cabin items
            for item in &cabin.items {
                if item.name().to_lowercase().contains(&normalized) {
                    return item.description().to_string();
                }
            }
        }
        Some(Room::WoodShed) => {
            if normalized.contains("block") || normalized.contains("chop") {
                let log_status = if wood_shed.chopping_block.has_log {
                    "A log sits ready on the chopping block."
                } else {
                    "The chopping block is empty, its surface scarred from countless cuts."
                };
                return log_status.to_string();
            }
            if normalized.contains("pile") || normalized.contains("stack") || normalized.contains("logs") {
                return format!(
                    "A stack of unsplit logs leans against the wall. You count {} logs remaining.",
                    wood_shed.logs
                );
            }
            if normalized.contains("axe") && wood_shed.axe_on_floor {
                return Item::Axe.description().to_string();
            }
        }
        _ => {}
    }

    // Self examination
    if normalized.contains("self") || normalized.contains("myself") || normalized == "me" {
        return player.status_summary();
    }

    format!("You don't see anything special about '{}'.", target)
}
