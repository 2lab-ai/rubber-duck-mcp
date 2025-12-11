use crate::entity::{Cabin, FireState, Item, Player, Room, Tree, TreeType, WoodShed};
use crate::persistence::GameState;
use crate::world::{Position, TileType, WorldMap};
use rand::Rng;

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
    map: &WorldMap,
    trees: &mut Vec<Tree>,
) -> CraftResult {
    let item = match Item::from_str(item_name) {
        Some(i) => i,
        None => return CraftResult::Failure(format!("You don't know what '{}' is.", item_name)),
    };

    // Check if player has the item (except for things they can use in place)
    let has_item = player.inventory.has(&item, 1);
    let target_normalized = target.as_ref().map(|s| s.to_lowercase());
    let target_str = target_normalized.as_deref();
    let target_is_fireplace = target_str
        .map(|t| t.contains("fire") || t.contains("hearth") || t.contains("fireplace"))
        .unwrap_or(false);
    let target_is_table = target_str
        .map(|t| t.contains("table") || t.contains("desk"))
        .unwrap_or(false);
    let in_cabin = matches!(player.room, Some(Room::CabinMain));

    if target_is_table && in_cabin {
        if !has_item {
            return CraftResult::Failure(format!("You don't have a {} to set down.", item.name()));
        }
        player.inventory.remove(&item, 1);
        cabin.add_table_item(item.clone());
        return CraftResult::Success(format!(
            "You place the {} carefully on the table.",
            item.name()
        ));
    }

    match (&item, target_str) {
        // Using axe on log (chopping)
        (Item::Axe, Some(t)) if t.contains("log") || t.contains("block") => {
            return try_chop(player, wood_shed);
        }

        // Using axe on standing tree
        (Item::Axe, Some(t)) if t.contains("tree") || t.contains("trunk") || t.contains("wood") => {
            return try_chop_tree(player, trees, map, wood_shed, cabin);
        }

        // Using axe on firewood to make kindling
        (Item::Axe, Some(t)) if t.contains("firewood") || t.contains("kindling") => {
            return try_split_firewood(player);
        }

        // Using log on chopping block
        (Item::Log, Some(t)) if t.contains("block") || t.contains("chop") => {
            return try_place_log(player, wood_shed);
        }

        // Using matchbox/match to light something
        (Item::Matchbox, Some(t)) if t.contains("kindling") => {
            return try_light_kindling(player, cabin);
        }
        (Item::Matchbox, _) if target_is_fireplace || (target_str.is_none() && in_cabin) => {
            return try_light_fire_with_match(player, cabin);
        }

        // Using lit kindling on fireplace
        (Item::LitKindling, _) if target_is_fireplace || (target_str.is_none() && in_cabin) => {
            return try_start_fire(player, cabin);
        }

        // Flammable items into the fireplace
        (i, _) if i.is_flammable() && target_is_fireplace => {
            return try_add_fuel_item(player, cabin, *i);
        }
        (i, None)
            if in_cabin
                && matches!(
                    i,
                    Item::Firewood
                        | Item::Kindling
                        | Item::Pinecone
                        | Item::Bark
                        | Item::DryLeaves
                        | Item::Driftwood
                        | Item::Log
                ) =>
        {
            return try_add_fuel_item(player, cabin, *i);
        }

        // Kettle: fill with water near the lake
        (Item::Kettle, Some(t))
            if t.contains("water") || t.contains("lake") || t.contains("fill") =>
        {
            return try_fill_kettle(player, map);
        }

        // Heat water in the kettle
        (Item::WaterKettle, Some(t))
            if t.contains("fire") || t.contains("hearth") || t.contains("boil") =>
        {
            return try_heat_kettle(player, cabin);
        }
        (Item::WaterKettle, None) if matches!(player.room, Some(Room::CabinMain)) => {
            return try_heat_kettle(player, cabin);
        }

        // Brew herbal tea
        (Item::WildHerbs, Some(t))
            if t.contains("tea")
                || t.contains("water")
                || t.contains("kettle")
                || t.contains("cup") =>
        {
            return try_brew_herbal_tea(player, cabin);
        }
        (Item::HotWaterKettle, Some(t))
            if t.contains("herb") || t.contains("tea") || t.contains("cup") =>
        {
            return try_brew_herbal_tea(player, cabin);
        }
        (Item::HotWaterKettle, None) if player.inventory.has(&Item::WildHerbs, 1) => {
            return try_brew_herbal_tea(player, cabin);
        }

        // Drink herbal tea
        (Item::HerbalTea, None) => {
            if has_item {
                return try_drink_herbal_tea(player);
            }
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

        // Eat berries
        (Item::WildBerry, None) => {
            if has_item {
                player.inventory.remove(&Item::WildBerry, 1);
                player.modify_fullness(8.0);
                player.modify_hydration(5.0);
                player.modify_energy(4.0);
                player.modify_mood(2.0);
                return CraftResult::Success(
                    "You eat the wild berries. Tart juice perks you up, and you feel a little less tired.".to_string()
                );
            }
        }

        // Eat apple
        (Item::Apple, None) => {
            if has_item {
                player.inventory.remove(&Item::Apple, 1);
                player.modify_fullness(15.0);
                player.modify_hydration(6.0);
                player.modify_energy(6.0);
                player.modify_mood(3.0);
                return CraftResult::Success(
                    "You bite into the apple. It's crisp and sweet, juices running over your tongue. You feel nourished.".to_string()
                );
            }
        }

        // Book -> kindling (tearing pages)
        (Item::OldBook, Some(t))
            if t.contains("kindling") || t.contains("tinder") || t.contains("burn") =>
        {
            return try_book_to_kindling(player);
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
                    "The cup is empty. You'd need water and a fire to make tea.".to_string(),
                );
            }
        }

        // Kettle with no target
        (Item::Kettle, None) => {
            if has_item {
                return CraftResult::Failure(
                    "The kettle is empty. Head to the lake to fill it with water.".to_string(),
                );
            }
        }

        // Herbs with no target
        (Item::WildHerbs, None) => {
            if has_item {
                if player.inventory.has(&Item::HotWaterKettle, 1) {
                    return CraftResult::Failure(
                        "You have hot water ready. Add the herbs to a cup with the kettle to steep tea.".to_string()
                    );
                }
                return CraftResult::Failure(
                    "The herbs smell fragrant. You'll need hot water and a cup to make tea."
                        .to_string(),
                );
            }
        }

        // Hot water with no target
        (Item::HotWaterKettle, None) => {
            if has_item {
                return CraftResult::Failure(
                    "The water is piping hot. Add herbs and pour it into a cup to make tea."
                        .to_string(),
                );
            }
        }

        _ => {}
    }

    if !has_item {
        return CraftResult::Failure(format!("You don't have a {}.", item.name()));
    }

    CraftResult::Failure(format!(
        "You're not sure how to use the {} {}.",
        item.name(),
        target.map(|t| format!("on '{}'", t)).unwrap_or_default()
    ))
}

pub fn kick_tree(state: &mut GameState) -> CraftResult {
    if state.player.room.is_some() {
        return CraftResult::Failure("You need to be outside near a tree to kick it.".to_string());
    }

    let Some(tree) = state.objects.find_tree_mut_at(&state.player.position) else {
        return CraftResult::Failure("There's no tree close enough to kick.".to_string());
    };

    if tree.felled {
        return CraftResult::Failure("That's already a felled tree.".to_string());
    }

    let mut rng = rand::thread_rng();
    let mut msg = String::from("You give the trunk a solid kick.");

    if tree.has_fruit() && rng.gen_bool(0.55) {
        let dropped = tree.take_fruit(1);
        if dropped > 0 {
            if let Some(fruit_item) = tree.fruit_item() {
                state.player.inventory.add(fruit_item, dropped as u32);
                msg.push_str(" A piece of fruit drops into your hands.");
            }
        }
    } else if rng.gen_bool(0.25) {
        msg.push_str(" The impact stings your toes, but the tree barely notices.");
        state.player.modify_mood(-1.0);
    } else {
        msg.push_str(" It shudders, scattering dust and bark.");
    }

    CraftResult::Success(msg)
}

/// Attempt to chop a log on the chopping block
fn try_chop(player: &mut Player, wood_shed: &mut WoodShed) -> CraftResult {
    // Must be in wood shed
    if !matches!(player.room, Some(Room::WoodShed)) {
        return CraftResult::Failure(
            "You need to be at the chopping block to chop wood.".to_string(),
        );
    }

    // Must have axe
    if !player.inventory.has(&Item::Axe, 1) {
        return CraftResult::Failure("You need to be holding an axe to chop wood.".to_string());
    }

    // Must have log on block
    if !wood_shed.chopping_block.has_log {
        return CraftResult::Failure(
            "There's no log on the chopping block. Place one first.".to_string(),
        );
    }

    // Skill check
    let skill = player.effective_skill("woodcutting");
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
        let _ = player.apply_body_damage(damage);

        CraftResult::PartialSuccess(format!(
            "The axe glances off at an awkward angle. You wince as the jarring impact sends pain through your arms. (-{:.1} health)",
            damage
        ))
    }
}

/// Place a log on the chopping block
fn try_place_log(player: &mut Player, wood_shed: &mut WoodShed) -> CraftResult {
    if !matches!(player.room, Some(Room::WoodShed)) {
        return CraftResult::Failure("The chopping block is in the wood shed.".to_string());
    }

    if wood_shed.chopping_block.has_log {
        return CraftResult::Failure("There's already a log on the chopping block.".to_string());
    }

    if !player.inventory.has(&Item::Log, 1) {
        return CraftResult::Failure("You don't have a log to place.".to_string());
    }

    player.inventory.remove(&Item::Log, 1);
    wood_shed.chopping_block.has_log = true;

    CraftResult::Success(
        "You heave the heavy log onto the chopping block and position it carefully.".to_string(),
    )
}

/// Light kindling with a match
fn try_light_kindling(player: &mut Player, _cabin: &Cabin) -> CraftResult {
    if !player.inventory.has(&Item::Matchbox, 1) {
        return CraftResult::Failure("You need a matchbox to light anything.".to_string());
    }

    if !player.inventory.has(&Item::Kindling, 1) {
        return CraftResult::Failure("You don't have any kindling to light.".to_string());
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
            "You need to be by the fireplace to start a fire.".to_string(),
        );
    }

    if !player.inventory.has(&Item::LitKindling, 1) {
        return CraftResult::Failure("You need lit kindling to start a fire.".to_string());
    }

    player.inventory.remove(&Item::LitKindling, 1);
    if !cabin.fireplace.add_fuel_item(Item::LitKindling) {
        player.inventory.add(Item::LitKindling, 1);
        return CraftResult::Failure(
            "The fireplace refuses to catch the burning kindling.".to_string(),
        );
    }

    attempt_light_fire(
        player,
        cabin,
        15.0,
        "You set the burning kindling into the hearth and shelter it from drafts.",
    )
}

fn try_light_fire_with_match(player: &mut Player, cabin: &mut Cabin) -> CraftResult {
    if !player.inventory.has(&Item::Matchbox, 1) {
        return CraftResult::Failure("You need your matchbox to strike a flame.".to_string());
    }

    attempt_light_fire(
        player,
        cabin,
        5.0,
        "You strike a match and touch it to the prepared tinder.",
    )
}

fn attempt_light_fire(
    player: &mut Player,
    cabin: &mut Cabin,
    bonus: f32,
    action_line: &str,
) -> CraftResult {
    if !matches!(player.room, Some(Room::CabinMain)) {
        return CraftResult::Failure("You need to be by the fireplace to do that.".to_string());
    }

    if cabin.fireplace.state != FireState::Cold {
        return CraftResult::Failure("The fire is already lit.".to_string());
    }

    if cabin.fireplace.fuel < 5.0 {
        return CraftResult::Failure(
            "There's not enough fuel in the fireplace. Add firewood, bark, leaves, pinecones, or another fuel source first.".to_string()
        );
    }

    if !cabin.fireplace.tinder_ready {
        return CraftResult::Failure(
            "You need tinder laid in the hearth before lighting (kindling, bark, dry leaves, book pages, pinecones).".to_string()
        );
    }

    let skill = player.effective_skill("fire_making") as f32;
    let base_chance = 50.0 + bonus;
    let success_chance = ((base_chance + skill * 0.5) / 100.0).min(0.95);

    let mut rng = rand::thread_rng();
    if rng.gen::<f32>() < success_chance {
        cabin.fireplace.state = FireState::Smoldering;
        cabin.fireplace.clear_tinder();

        if rng.gen_bool(0.3) {
            player.skills.improve("fire_making", 1);
        }

        CraftResult::Success(format!(
            "{} The tinder catches and the larger fuel begins to smolder.",
            action_line
        ))
    } else {
        cabin.fireplace.clear_tinder();
        cabin.fireplace.fuel = (cabin.fireplace.fuel - 2.0).max(0.0);

        CraftResult::PartialSuccess(
            "The flames gasp and die before the larger fuel takes. Add fresh tinder and try again."
                .to_string(),
        )
    }
}

fn try_add_fuel_item(player: &mut Player, cabin: &mut Cabin, item: Item) -> CraftResult {
    if !matches!(player.room, Some(Room::CabinMain)) {
        return CraftResult::Failure("You need to be by the fireplace.".to_string());
    }

    if !item.is_flammable() {
        return CraftResult::Failure(format!("The {} won't help the fire.", item.name()));
    }

    if !player.inventory.has(&item, 1) {
        return CraftResult::Failure(format!("You don't have any {} to add.", item.name()));
    }

    player.inventory.remove(&item, 1);
    if !cabin.fireplace.add_fuel_item(item) {
        player.inventory.add(item, 1);
        return CraftResult::Failure(format!(
            "The {} refuses to catch in the fireplace.",
            item.name()
        ));
    }

    let mut msg = if cabin.fireplace.state == FireState::Cold {
        format!("You set the {} into the fireplace.", item.name())
    } else {
        format!("You feed the flames with the {}.", item.name())
    };

    if item.is_tinder() && cabin.fireplace.state == FireState::Cold {
        msg.push_str(" It should help the fire catch quickly when you strike a match.");
    }

    CraftResult::Success(msg)
}

/// Turn a piece of firewood into kindling
fn try_split_firewood(player: &mut Player) -> CraftResult {
    if !player.inventory.has(&Item::Firewood, 1) {
        return CraftResult::Failure("You don't have any firewood to split.".to_string());
    }

    player.inventory.remove(&Item::Firewood, 1);
    let mut rng = rand::thread_rng();
    let bundles = rng.gen_range(2..=3);
    // Firewood weight (1.5) -> kindling bundles (0.3 each) so always lighter
    player.inventory.add(Item::Kindling, bundles);
    player.modify_energy(-2.0);

    CraftResult::Success(
        format!(
            "You shave down a piece of firewood with careful axe strokes, producing {} bundles of fine kindling shavings.",
            bundles
        )
    )
}

/// Tear pages from the old book to use as kindling
fn try_book_to_kindling(player: &mut Player) -> CraftResult {
    if !player.inventory.has(&Item::OldBook, 1) {
        return CraftResult::Failure("You don't have the old book with you.".to_string());
    }

    player.inventory.remove(&Item::OldBook, 1);
    player.inventory.add(Item::Kindling, 1);
    player.modify_mood(-1.0);

    CraftResult::Success(
        "With a wince, you tear out a handful of brittle pages and crumple them. They should catch a flame easily, even if it feels a little wrong.".to_string()
    )
}

/// Find a nearby standing tree
fn find_near_tree<'a>(player: &Player, trees: &'a mut [Tree]) -> Option<&'a mut Tree> {
    trees
        .iter_mut()
        .find(|t| !t.felled && player.position.distance_to(&t.position) <= 1.5)
}

fn collect_fruit_drop(
    tree: &mut Tree,
    player: &mut Player,
    cabin: &mut Cabin,
    count: u8,
) -> Option<String> {
    let Some(item) = tree.fruit_item() else {
        return None;
    };
    let taken = tree.take_fruit(count);
    if taken == 0 {
        return None;
    }

    let mut caught = 0;
    for _ in 0..taken {
        if player.inventory.add(item.clone(), 1) {
            caught += 1;
        } else {
            cabin.add_item(item.clone());
        }
    }

    if caught > 0 {
        Some(format!("A {} drops and you grab it.", item.name()))
    } else {
        Some(format!(
            "A {} thuds to the ground. You can pick it up later.",
            item.name()
        ))
    }
}

/// Chop a standing tree in the forest
fn try_chop_tree(
    player: &mut Player,
    trees: &mut Vec<Tree>,
    map: &WorldMap,
    wood_shed: &mut WoodShed,
    cabin: &mut Cabin,
) -> CraftResult {
    if player.room.is_some() {
        return CraftResult::Failure("You need to be outside near a tree.".to_string());
    }

    // Make sure there is a tree nearby
    let tree = match find_near_tree(player, trees) {
        Some(t) => t,
        None => {
            return CraftResult::Failure("You don't see a tree close enough to chop.".to_string())
        }
    };

    // Guard against non-walkable tiles (just in case)
    if let Some((row, col)) = tree.position.as_usize() {
        if !map.is_walkable(row, col) {
            return CraftResult::Failure("You can't reach that tree from here.".to_string());
        }
    }

    player.modify_energy(-6.0);

    let skill = player.effective_skill("woodcutting");
    let base = 55.0;
    let success_chance = ((base + skill as f32 * 0.4) / 100.0).min(0.95);

    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();
    let mut fruit_note = collect_fruit_drop(tree, player, cabin, 1);

    if roll > success_chance {
        let damage = rng.gen_range(1.0..3.0);
        let _ = player.apply_body_damage(damage);
        let mut msg = format!(
            "Your swing glances off the trunk, jarring your arms (-{:.1} health). {}",
            damage,
            tree.progress_text()
        );
        if let Some(note) = fruit_note.take() {
            msg.push(' ');
            msg.push_str(&note);
        }
        return CraftResult::PartialSuccess(msg);
    }

    tree.hits_done = (tree.hits_done + 1).min(tree.hits_required);
    if rng.gen_bool(0.25) {
        player.skills.improve("woodcutting", 1);
    }

    if tree.hits_done >= tree.hits_required {
        tree.felled = true;
        // Decide drops
        let (logs, kindling, pinecones, apples, bark, leaves) = match tree.kind {
            TreeType::Pine => (
                rng.gen_range(2..=3),
                rng.gen_range(2..=3),
                rng.gen_range(1..=3),
                0,
                rng.gen_range(1..=2),
                rng.gen_range(0..=2),
            ),
            TreeType::Birch => (
                rng.gen_range(2..=4),
                rng.gen_range(1..=2),
                0,
                0,
                rng.gen_range(2..=3),
                rng.gen_range(1..=2),
            ),
            TreeType::Apple => (
                rng.gen_range(1..=3),
                rng.gen_range(1..=2),
                0,
                rng.gen_range(1..=3),
                rng.gen_range(1..=2),
                rng.gen_range(1..=3),
            ),
            TreeType::Bamboo => (0, 0, 0, 0, rng.gen_range(1..=2), rng.gen_range(1..=2)),
        };

        let mut carried = Vec::new();
        let mut stashed = Vec::new();
        let mut felled_fruit = 0;

        // Helper to add or stash items
        let mut add_or_stash = |item: Item, qty: u32| {
            for _ in 0..qty {
                if player.inventory.add(item.clone(), 1) {
                    carried.push(item.clone());
                } else {
                    match item {
                        Item::Log => {
                            wood_shed.logs += 1;
                            stashed.push("log to wood shed");
                        }
                        Item::Kindling => {
                            cabin.add_item(Item::Kindling);
                            stashed.push("kindling in cabin");
                        }
                        Item::Pinecone => {
                            cabin.add_item(Item::Pinecone);
                            stashed.push("pinecone in cabin");
                        }
                        Item::Bark => {
                            cabin.add_item(Item::Bark);
                            stashed.push("bark in cabin");
                        }
                        Item::DryLeaves => {
                            cabin.add_item(Item::DryLeaves);
                            stashed.push("dry leaves in cabin");
                        }
                        _ => {}
                    }
                }
            }
        };

        if logs > 0 {
            add_or_stash(Item::Log, logs);
        }
        if kindling > 0 {
            add_or_stash(Item::Kindling, kindling);
        }
        if pinecones > 0 {
            add_or_stash(Item::Pinecone, pinecones);
        }
        if apples > 0 {
            add_or_stash(Item::Apple, apples);
        }
        if bark > 0 {
            add_or_stash(Item::Bark, bark);
        }
        if leaves > 0 {
            add_or_stash(Item::DryLeaves, leaves);
        }
        if let Some(item) = tree.fruit_item() {
            felled_fruit = tree.take_all_fruit() as u32;
            if felled_fruit > 0 {
                add_or_stash(item, felled_fruit);
            }
        }

        if matches!(tree.kind, TreeType::Bamboo) {
            add_or_stash(Item::Bamboo, rng.gen_range(2..=4));
            return CraftResult::Success(
                "With a final swing, the bamboo stalks topple.".to_string(),
            );
        }

        let mut summary = format!(
            "With a final swing, the {} creaks and crashes down. You gain {} logs and {} bundles of kindling.",
            match tree.kind {
                TreeType::Pine => "pine tree",
                TreeType::Birch => "birch tree",
                TreeType::Apple => "apple tree",
                TreeType::Bamboo => "bamboo stand",
            },
            logs,
            kindling
        );
        if pinecones > 0 {
            summary.push_str(&format!(" Pinecones collected: {}.", pinecones));
        }
        if apples > 0 {
            summary.push_str(&format!(" Apples gathered: {}.", apples));
        }
        if felled_fruit > 0 {
            summary.push_str(&format!(" Extra fruit shaken loose: {}.", felled_fruit));
        }
        if bark > 0 {
            summary.push_str(&format!(" Bark strips gathered: {}.", bark));
        }
        if leaves > 0 {
            summary.push_str(&format!(" Dry leaves collected: {}.", leaves));
        }
        if !stashed.is_empty() {
            summary.push_str(" Some heavy items were stashed: ");
            summary.push_str(&stashed.join(", "));
            summary.push('.');
        }
        if !carried.is_empty() {
            summary.push_str(" You tuck the rest into your pack.");
        }

        CraftResult::Success(summary)
    } else {
        let mut msg = format!(
            "You sink the axe into the trunk. Chips fly. {}",
            tree.progress_text()
        );
        if let Some(note) = fruit_note.take() {
            msg.push(' ');
            msg.push_str(&note);
        }
        CraftResult::Success(msg)
    }
}

/// Check if the player is adjacent to water they can fill the kettle from
fn is_near_water(player: &Player, map: &WorldMap) -> bool {
    let pr = player.position.row;
    let pc = player.position.col;
    for dr in -1..=1 {
        for dc in -1..=1 {
            let pos = Position::new(pr + dr, pc + dc);
            if !pos.is_valid() {
                continue;
            }
            if let Some((r, c)) = pos.as_usize() {
                if let Some(tile) = map.get_tile(r, c) {
                    if matches!(tile.tile_type, TileType::Lake) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Fill the kettle with lake water
fn try_fill_kettle(player: &mut Player, map: &WorldMap) -> CraftResult {
    if !player.inventory.has(&Item::Kettle, 1) {
        return CraftResult::Failure("You don't have a kettle to fill.".to_string());
    }

    if player.is_indoor() {
        return CraftResult::Failure(
            "You'll need to step outside or onto the terrace to fetch water.".to_string(),
        );
    }

    if !is_near_water(player, map) {
        return CraftResult::Failure(
            "You need to be right by the lake to fill the kettle.".to_string(),
        );
    }

    let extra_weight = Item::WaterKettle.weight() - Item::Kettle.weight();
    if player.inventory.current_weight() + extra_weight > player.inventory.max_weight {
        return CraftResult::Failure(
            "A kettle full of water would be too heavy for you to carry right now.".to_string(),
        );
    }

    player.inventory.remove(&Item::Kettle, 1);
    player.inventory.add(Item::WaterKettle, 1);

    CraftResult::Success(
        "You dip the kettle into the lake and scoop up water. It's a bit murky — better boil it."
            .to_string(),
    )
}

/// Heat a kettle of water over the fireplace
fn try_heat_kettle(player: &mut Player, cabin: &Cabin) -> CraftResult {
    if !player.inventory.has(&Item::WaterKettle, 1) {
        return CraftResult::Failure("You need a kettle filled with water first.".to_string());
    }

    if !matches!(player.room, Some(Room::CabinMain)) {
        return CraftResult::Failure(
            "You need to set the kettle by the fireplace in the cabin.".to_string(),
        );
    }

    if cabin.fireplace.state == FireState::Cold {
        return CraftResult::Failure(
            "The hearth is cold. Get a fire going before trying to boil water.".to_string(),
        );
    }

    player.inventory.remove(&Item::WaterKettle, 1);
    player.inventory.add(Item::Kettle, 1);
    player.inventory.add(Item::CleanWater, 1);

    CraftResult::Success(
        "You set the kettle near the flames. Soon it begins to murmur and steam. You pour out clean, boiled water.".to_string()
    )
}

/// Brew herbal tea using hot water, herbs, and a cup
fn try_brew_herbal_tea(player: &mut Player, _cabin: &Cabin) -> CraftResult {
    if !matches!(player.room, Some(Room::CabinMain)) {
        return CraftResult::Failure(
            "Find a steady spot by the cabin hearth to brew your tea.".to_string(),
        );
    }

    let has_hot = player.inventory.has(&Item::HotWaterKettle, 1);
    let has_clean = player.inventory.has(&Item::CleanWater, 1);
    if !has_hot && !has_clean {
        return CraftResult::Failure(
            "You need clean, hot water to steep the herbs. Boil lake water first.".to_string(),
        );
    }

    if !player.inventory.has(&Item::TeaCup, 1) {
        return CraftResult::Failure("You'll need a cup ready to pour the tea into.".to_string());
    }

    if !player.inventory.has(&Item::WildHerbs, 1) {
        return CraftResult::Failure("You don't have any wild herbs to steep.".to_string());
    }

    if has_hot {
        player.inventory.remove(&Item::HotWaterKettle, 1);
        player.inventory.add(Item::Kettle, 1);
    } else {
        player.inventory.remove(&Item::CleanWater, 1);
    }
    player.inventory.remove(&Item::WildHerbs, 1);
    player.inventory.remove(&Item::TeaCup, 1);
    player.inventory.add(Item::HerbalTea, 1);

    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.25) {
        player.skills.improve("foraging", 1);
    }

    CraftResult::Success(
        "You add the fragrant herbs to your cup and pour in the hot water. Steam curls upward, carrying notes of mint and chamomile. The tea needs a moment to steep.".to_string()
    )
}

/// Drink a cup of herbal tea
fn try_drink_herbal_tea(player: &mut Player) -> CraftResult {
    if !player.inventory.has(&Item::HerbalTea, 1) {
        return CraftResult::Failure("You don't have any tea to drink.".to_string());
    }

    player.inventory.remove(&Item::HerbalTea, 1);
    player.inventory.add(Item::TeaCup, 1);
    player.modify_warmth(12.0);
    player.modify_mood(18.0);
    player.modify_energy(6.0);

    CraftResult::Success(
        "You sip the warm herbal tea. A sense of calm washes over you as the heat spreads through your chest. You're getting better at this little ritual.".to_string()
    )
}
