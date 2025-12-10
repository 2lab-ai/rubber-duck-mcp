use rand::Rng;
use crate::entity::{Player, Room, Item, Cabin, WoodShed, FireState};

pub enum CraftResult {
    Success(String),
    Failure(String),
    PartialSuccess(String),
}

/// Use an item, optionally on a target
pub fn use_item(
    item_name: &str,
    target: Option<&str>,
    player: &mut Player,
    cabin: &mut Cabin,
    wood_shed: &mut WoodShed,
) -> CraftResult {
    let item = match Item::from_str(item_name) {
        Some(i) => i,
        None => return CraftResult::Failure(format!("You don't know what '{}' is.", item_name)),
    };

    // Check if player has the item (except for things they can use in place)
    let has_item = player.inventory.has(&item, 1);

    match (&item, target.map(|s| s.to_lowercase()).as_deref()) {
        // Using axe on log (chopping)
        (Item::Axe, Some(t)) if t.contains("log") || t.contains("block") => {
            return try_chop(player, wood_shed);
        }

        // Using log on chopping block
        (Item::Log, Some(t)) if t.contains("block") || t.contains("chop") => {
            return try_place_log(player, wood_shed);
        }

        // Using matchbox/match to light something
        (Item::Matchbox, Some(t)) if t.contains("kindling") => {
            return try_light_kindling(player, cabin);
        }
        (Item::Matchbox, Some(t)) if t.contains("fire") || t.contains("hearth") => {
            // Try to light fireplace directly (will fail without kindling)
            return CraftResult::Failure(
                "You can't light the fireplace directly. Try lighting some kindling first.".to_string()
            );
        }

        // Using lit kindling on fireplace
        (Item::LitKindling, Some(t)) if t.contains("fire") || t.contains("hearth") => {
            return try_start_fire(player, cabin);
        }
        (Item::LitKindling, None) if matches!(player.room, Some(Room::CabinMain)) => {
            // Default to fireplace if in cabin
            return try_start_fire(player, cabin);
        }

        // Using firewood on fireplace (adding fuel)
        (Item::Firewood, Some(t)) if t.contains("fire") || t.contains("hearth") => {
            return try_add_fuel(player, cabin);
        }
        (Item::Firewood, None) if matches!(player.room, Some(Room::CabinMain)) => {
            return try_add_fuel(player, cabin);
        }

        // Using kindling on fireplace (prepare for lighting)
        (Item::Kindling, Some(t)) if t.contains("fire") || t.contains("hearth") => {
            if !has_item {
                return CraftResult::Failure("You don't have any kindling.".to_string());
            }
            // Just a message - kindling should be lit first
            return CraftResult::Failure(
                "You should light the kindling before putting it in the fireplace. Use the matchbox on the kindling.".to_string()
            );
        }

        // Blanket - warm up
        (Item::WoolBlanket, None) => {
            if has_item {
                player.modify_warmth(10.0);
                player.modify_mood(5.0);
                return CraftResult::Success(
                    "You wrap the thick wool blanket around your shoulders. Its warmth is immediately comforting.".to_string()
                );
            }
        }

        // Book - read it
        (Item::OldBook, None) => {
            if has_item {
                player.modify_mood(3.0);
                return CraftResult::Success(
                    "You flip through the ancient pages. The symbols are indecipherable, but the intricate diagrams of celestial bodies are fascinating.".to_string()
                );
            }
        }

        // Compass - check it
        (Item::StrangeCompass, None) => {
            if has_item {
                return CraftResult::Success(
                    "The needle spins lazily, then settles pointing toward the center of the lake. Curious.".to_string()
                );
            }
        }

        // Map - view it
        (Item::AncientMap, None) => {
            if has_item {
                return CraftResult::Success(
                    "You unfold the brittle map. It shows this very area - but different. The lake is labeled 'The Mirror', and something is marked at its center.".to_string()
                );
            }
        }

        // Tea cup with no target
        (Item::TeaCup, None) => {
            if has_item {
                return CraftResult::Failure(
                    "The cup is empty. You'd need water and a fire to make tea.".to_string()
                );
            }
        }

        _ => {}
    }

    if !has_item {
        return CraftResult::Failure(format!("You don't have a {}.", item.name()));
    }

    CraftResult::Failure(
        format!("You're not sure how to use the {} {}.",
            item.name(),
            target.map(|t| format!("on '{}'", t)).unwrap_or_default()
        )
    )
}

/// Attempt to chop a log on the chopping block
fn try_chop(player: &mut Player, wood_shed: &mut WoodShed) -> CraftResult {
    // Must be in wood shed
    if !matches!(player.room, Some(Room::WoodShed)) {
        return CraftResult::Failure(
            "You need to be at the chopping block to chop wood.".to_string()
        );
    }

    // Must have axe
    if !player.inventory.has(&Item::Axe, 1) {
        return CraftResult::Failure(
            "You need to be holding an axe to chop wood.".to_string()
        );
    }

    // Must have log on block
    if !wood_shed.chopping_block.has_log {
        return CraftResult::Failure(
            "There's no log on the chopping block. Place one first.".to_string()
        );
    }

    // Skill check
    let skill = player.skills.get("woodcutting");
    let base_chance = 50.0;
    let skill_bonus = skill as f32 / 2.0;
    let success_chance = (base_chance + skill_bonus) / 100.0;

    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();

    // Energy cost
    player.modify_energy(-5.0);

    if roll < success_chance {
        // Success!
        wood_shed.chopping_block.has_log = false;

        // Generate 2-4 pieces of firewood
        let pieces = rng.gen_range(2..=4);
        wood_shed.firewood += pieces;

        // Skill improvement
        if rng.gen_bool(0.3) {
            player.skills.improve("woodcutting", 1);
        }

        CraftResult::Success(format!(
            "THWACK! The axe bites deep and the log splits cleanly. You've produced {} pieces of firewood!",
            pieces
        ))
    } else {
        // Failure
        let damage = rng.gen_range(1.0..5.0);
        player.modify_health(-damage);

        CraftResult::PartialSuccess(format!(
            "The axe glances off at an awkward angle. You wince as the jarring impact sends pain through your arms. (-{:.1} health)",
            damage
        ))
    }
}

/// Place a log on the chopping block
fn try_place_log(player: &mut Player, wood_shed: &mut WoodShed) -> CraftResult {
    if !matches!(player.room, Some(Room::WoodShed)) {
        return CraftResult::Failure(
            "The chopping block is in the wood shed.".to_string()
        );
    }

    if wood_shed.chopping_block.has_log {
        return CraftResult::Failure(
            "There's already a log on the chopping block.".to_string()
        );
    }

    if !player.inventory.has(&Item::Log, 1) {
        return CraftResult::Failure(
            "You don't have a log to place.".to_string()
        );
    }

    player.inventory.remove(&Item::Log, 1);
    wood_shed.chopping_block.has_log = true;

    CraftResult::Success(
        "You heave the heavy log onto the chopping block and position it carefully.".to_string()
    )
}

/// Light kindling with a match
fn try_light_kindling(player: &mut Player, _cabin: &Cabin) -> CraftResult {
    if !player.inventory.has(&Item::Matchbox, 1) {
        return CraftResult::Failure(
            "You need a matchbox to light anything.".to_string()
        );
    }

    if !player.inventory.has(&Item::Kindling, 1) {
        return CraftResult::Failure(
            "You don't have any kindling to light.".to_string()
        );
    }

    // Convert kindling to lit kindling
    player.inventory.remove(&Item::Kindling, 1);
    player.inventory.add(Item::LitKindling, 1);

    CraftResult::Success(
        "You strike a match against the box. It flares to life with a hiss. You touch it to the kindling, which catches fire, flames dancing eagerly. You'd better use it quickly!".to_string()
    )
}

/// Start a fire in the fireplace with lit kindling
fn try_start_fire(player: &mut Player, cabin: &mut Cabin) -> CraftResult {
    if !matches!(player.room, Some(Room::CabinMain)) {
        return CraftResult::Failure(
            "You need to be by the fireplace to start a fire.".to_string()
        );
    }

    if !player.inventory.has(&Item::LitKindling, 1) {
        return CraftResult::Failure(
            "You need lit kindling to start a fire.".to_string()
        );
    }

    if cabin.fireplace.fuel < 5.0 {
        player.inventory.remove(&Item::LitKindling, 1);
        return CraftResult::Failure(
            "There's not enough fuel in the fireplace. The kindling burns out uselessly. You should add firewood first.".to_string()
        );
    }

    if cabin.fireplace.state != FireState::Cold {
        return CraftResult::Failure(
            "The fire is already lit.".to_string()
        );
    }

    // Skill check
    let skill = player.skills.get("fire_making");
    let base_chance = 40.0;
    let skill_bonus = skill as f32 / 2.0;
    let success_chance = (base_chance + skill_bonus) / 100.0;

    let mut rng = rand::thread_rng();
    player.inventory.remove(&Item::LitKindling, 1);

    if rng.gen::<f32>() < success_chance {
        cabin.fireplace.state = FireState::Smoldering;

        // Skill improvement
        if rng.gen_bool(0.3) {
            player.skills.improve("fire_making", 1);
        }

        CraftResult::Success(
            "You carefully place the burning kindling among the firewood. The flames spread, licking at the larger pieces. Smoke rises as the fire takes hold. Success!".to_string()
        )
    } else {
        CraftResult::PartialSuccess(
            "You place the kindling, but the flames sputter and die before catching the larger wood. The kindling is consumed. You'll need to try again.".to_string()
        )
    }
}

/// Add fuel to an existing fire
fn try_add_fuel(player: &mut Player, cabin: &mut Cabin) -> CraftResult {
    if !matches!(player.room, Some(Room::CabinMain)) {
        return CraftResult::Failure(
            "You need to be by the fireplace.".to_string()
        );
    }

    if !player.inventory.has(&Item::Firewood, 1) {
        return CraftResult::Failure(
            "You don't have any firewood to add.".to_string()
        );
    }

    player.inventory.remove(&Item::Firewood, 1);

    let fuel_value = Item::Firewood.fuel_value().unwrap_or(30.0);
    cabin.fireplace.add_fuel(fuel_value);

    let msg = if cabin.fireplace.state == FireState::Cold {
        "You stack the firewood in the fireplace. Now you just need to light it."
    } else {
        "You add firewood to the fire. The flames eagerly consume the new fuel."
    };

    CraftResult::Success(msg.to_string())
}
