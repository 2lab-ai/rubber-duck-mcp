use crate::entity::{Blueprint, BookEntry, FireState, Item, Room};
use crate::persistence::GameState;
use crate::world::{Biome, Position, TimeOfDay, Weather, WorldMap};
use rand::Rng;

pub enum InteractionResult {
    Success(String),
    Failure(String),
    ItemObtained(Item, String),
    ItemLost(Item, String),
    ActionSuccess {
        message: String,
        time_cost: u32,
        energy_cost: f32,
    },
}

// ... (Duck constants omit for brevity, will include) ...
const DUCK_GAZE: &[&str] = &[
    "The rubber duck fixes you with a glassy stare.",
    "The duck's eyes seem to track your words.",
    "The duck tilts ever so slightly, as if curious.",
    "It sits motionless, yet attentive.",
    "The duck seems to regard you as a puzzle.",
    "Its painted eyes look ancient for a toy.",
    "It leans into the silence as if absorbing it.",
    "You swear it blinks, though you know it cannot.",
    "The duck looks as if it has heard this before.",
    "It seems to nod, or maybe that's your imagination.",
    "Its beak gleams as though poised to speak.",
    "The duck's gaze drifts beyond you, pondering.",
    "It appears to be weighing possibilities.",
    "Its tiny eyes flick side to side thoughtfully.",
    "It seems to follow an invisible thought map.",
    "The duck squares its tiny shoulders solemnly.",
    "Its stare softens, almost compassionate.",
    "It regards you like an old confidant.",
    "Its eyes widen, then settle back.",
    "You feel seen, somehow, by plastic eyes.",
    "The duck looks patient—like it has all night.",
    "It absorbs your words like a sponge.",
    "The duck fixes on the middle distance.",
    "It rocks imperceptibly in contemplation.",
    "Its gaze sharpens, like a sage in miniature.",
    "It seems to weigh each syllable.",
    "You catch a hint of bemused curiosity.",
    "Its stare is unwavering, steady as bedrock.",
    "It leans forward, inviting more.",
    "The duck's eyes glint with mock wisdom.",
    "It seems to study you, cataloging data.",
    "The duck listens with improbable gravitas.",
    "Its eyes soften as if understanding.",
    "It appears to approve of your inquiry.",
    "The duck's blank face feels suddenly full.",
    "It looks up like a mentor expecting insight.",
    "Its gaze is unfathomable and kind.",
    "It radiates calm expectancy.",
    "The duck looks conspiratorial.",
    "It seems to hum without sound.",
    "Its stare drifts to some internal horizon.",
    "You feel as if questioned in return.",
    "The duck holds its silence like a vow.",
    "It leans into the moment, serene.",
    "Its eyes dart, cataloging unseen things.",
    "It wears the air of a patient teacher.",
    "The duck looks ready to annotate reality.",
    "Its stare is half-solemn, half-amused.",
    "It seems amused by your urgency.",
    "The duck appears to savor the question.",
    "Its gaze grows distant, then returns.",
    "You sense it filing your words away.",
];

const DUCK_MANNER: &[&str] = &[
    "It bobs once, barely noticeable.",
    "A slow, imaginary nod seems to happen.",
    "The duck tilts as if tasting the thought.",
    "A faint squeak almost emerges, then doesn't.",
    "You can almost hear gears turning inside its head.",
    "It holds perfectly still, like a monk at dawn.",
    "Its stillness grows louder than speech.",
    "It seems to inhale an invisible breath.",
    "A ripple of contemplation passes over it.",
    "Its plastic shell looks suddenly venerable.",
    "It leans toward you, eager yet mute.",
    "The duck seems to sift your words like tea leaves.",
    "It studies the floor as if answers hide there.",
    "Its head cants sideways, inquisitive.",
    "You sense it rehearsing a profound reply.",
    "A miniature frown seems to crease its brow.",
    "It appears to moult old assumptions.",
    "The duck gently rocks, weighing outcomes.",
    "Its silence stretches, thoughtful and warm.",
    "It emits a soft aura of patience.",
    "A ghost of a quack hovers in the air.",
    "Its beak parts slightly, then closes again.",
    "It traces invisible diagrams in the air.",
    "A hush wraps around the duck like a cloak.",
    "It looks at you, then at the horizon beyond.",
    "Its attention is total, undivided.",
    "It seems to file this under 'important'.",
    "It nods inwardly, as if agreeing with itself.",
    "It appears to highlight a passage in an unseen book.",
    "It pauses, as if letting your words breathe.",
    "It radiates a question back at you.",
    "It seems to underline an unspoken lesson.",
    "The duck gently sways, like a scholar in thought.",
    "It absorbs the silence like sunlight.",
    "It looks past you, toward some broader truth.",
    "A tiny sigh you imagine echoes faintly.",
    "It slow-blinks with invisible eyelids.",
    "It gestures minutely toward your heart.",
    "The duck seems to quote an unwritten poem.",
    "It arranges your words in an invisible stack.",
    "It glances at an inner chalkboard.",
    "It weighs paradoxes like pebbles.",
    "The duck squints inwardly at a dilemma.",
    "It looks as if it forgives the universe.",
    "It leans back, bathing in the question.",
    "Its posture says 'go on' without sound.",
    "It cups silence in its little wings.",
    "It seems to practice saying nothing perfectly.",
    "Its focus is a lantern in the dim room.",
    "It quietly invites you to fill the silence.",
    "It seems to rehearse a koan.",
    "It smiles without moving.",
];

fn random_duck_phrase(rng: &mut impl rand::Rng) -> String {
    use rand::seq::SliceRandom;
    let part_a = DUCK_GAZE
        .choose(rng)
        .unwrap_or(&"The rubber duck is very present.");
    let part_b = DUCK_MANNER.choose(rng).unwrap_or(&"It stays very still.");
    format!("{} {}", part_a, part_b)
}

// ... Open/Close/Take/Drop handlers (omitted here to save space if unchanged, but will include needed ones) ...
// Actually, I need to include them to overwrite the file properly.

pub fn try_open(target: &str, state: &mut GameState) -> InteractionResult {
    let normalized = target.to_lowercase();
    if normalized.contains("card") || normalized.contains("case") {
        if !state.player.inventory.has(&Item::CardCase, 1) {
            return InteractionResult::Failure(
                "You need to be holding the card case to open it.".to_string(),
            );
        }
        if state.card_case_open {
            return InteractionResult::Failure("The card case is already open.".to_string());
        }
        state.card_case_open = true;
        return InteractionResult::Success(
            "You flip open the card case. A neat stack of cards waits inside."
                .to_string(),
        );
    }

    let cabin_pos = match state.objects.find("cabin") {
        Some(obj) => obj.position,
        None => return InteractionResult::Failure("You don't see a cabin to open.".to_string()),
    };
    if normalized.contains("door") || normalized.contains("cabin") {
        let near_cabin = {
            let room = state.player.room.clone();
            state.player.position.distance_to(&cabin_pos) <= 1.5
                || matches!(room, Some(Room::CabinMain))
        };
        let Some(cabin) = state.cabin_state_mut() else {
            return InteractionResult::Failure("The cabin seems missing its details.".to_string());
        };
        if cabin.door_open {
            return InteractionResult::Failure("The door is already open.".to_string());
        }
        if !near_cabin {
            return InteractionResult::Failure("You're too far from the cabin door.".to_string());
        }
        cabin.door_open = true;
        InteractionResult::Success(
            "You grasp the worn iron handle and pull. The door swings open with a soft creak."
                .to_string(),
        )
    } else {
        InteractionResult::Failure(format!("You don't see a '{}' to open.", target))
    }
}

pub fn try_close(target: &str, state: &mut GameState) -> InteractionResult {
    let normalized = target.to_lowercase();
    if normalized.contains("card") || normalized.contains("case") {
        if !state.player.inventory.has(&Item::CardCase, 1) {
            return InteractionResult::Failure(
                "You need to be holding the card case to close it.".to_string(),
            );
        }
        if !state.card_case_open {
            return InteractionResult::Failure("The card case is already closed.".to_string());
        }
        state.card_case_open = false;
        return InteractionResult::Success(
            "You snap the card case shut, keeping the deck contained."
                .to_string(),
        );
    }

    let cabin_pos = match state.objects.find("cabin") {
        Some(obj) => obj.position,
        None => return InteractionResult::Failure("You don't see a cabin to close.".to_string()),
    };
    if normalized.contains("door") || normalized.contains("cabin") {
        let near_cabin = {
            let room = state.player.room.clone();
            state.player.position.distance_to(&cabin_pos) <= 1.5
                || matches!(room, Some(Room::CabinMain))
        };
        let Some(cabin) = state.cabin_state_mut() else {
            return InteractionResult::Failure("The cabin seems missing its details.".to_string());
        };
        if !cabin.door_open {
            return InteractionResult::Failure("The door is already closed.".to_string());
        }
        if !near_cabin {
            return InteractionResult::Failure("You're too far from the cabin door.".to_string());
        }
        cabin.door_open = false;
        InteractionResult::Success(
            "You push the door closed. It latches with a satisfying click.".to_string(),
        )
    } else {
        InteractionResult::Failure(format!("You don't see a '{}' to close.", target))
    }
}

pub fn try_take(item_name: &str, state: &mut GameState, map: &mut WorldMap) -> InteractionResult {
    let item = match Item::from_str(item_name) {
        Some(i) => i,
        None => {
            return InteractionResult::Failure(format!("You don't know what '{}' is.", item_name))
        }
    };

    let player_room = state.player.room.clone();

    match player_room {
        Some(Room::CabinMain) => {
            let mut from_cabin_floor = false;
            {
                if let Some(cabin) = state.cabin_state_mut() {
                    from_cabin_floor = cabin.take_item(&item);
                }
            }
            if from_cabin_floor {
                if state.player.inventory.add(item.clone(), 1) {
                    state.on_player_pickup(&item);
                    return InteractionResult::ItemObtained(
                        item.clone(),
                        format!("You pick up the {}.", item.name()),
                    );
                } else {
                    if let Some(cabin) = state.cabin_state_mut() {
                        cabin.add_item(item.clone());
                    }
                    return InteractionResult::Failure("Your inventory is too heavy.".to_string());
                }
            }

            if state.take_table_item(&item) {
                if state.player.inventory.add(item.clone(), 1) {
                    state.on_player_pickup(&item);
                    return InteractionResult::ItemObtained(
                        item.clone(),
                        format!("You lift the {} from the table.", item.name()),
                    );
                } else {
                    state.add_table_item(item.clone());
                    return InteractionResult::Failure("Too heavy.".to_string());
                }
            }

            if item == Item::Matchbox {
                let mut took = false;
                {
                    if let Some(cabin) = state.cabin_state_mut() {
                        if cabin.has_item(&Item::Matchbox) {
                            took = cabin.take_item(&Item::Matchbox);
                        }
                    }
                }
                if took && state.player.inventory.add(Item::Matchbox, 1) {
                    state.on_player_pickup(&Item::Matchbox);
                    return InteractionResult::ItemObtained(
                        Item::Matchbox,
                        "You take the matchbox from the mantelpiece.".to_string(),
                    );
                } else if took {
                    if let Some(cabin) = state.cabin_state_mut() {
                        cabin.add_item(Item::Matchbox);
                    }
                    return InteractionResult::Failure("Your inventory is too heavy.".to_string());
                }
            }
        }
        Some(Room::WoodShed) => {
            if item == Item::Axe {
                let mut picked_up = false;
                {
                    if let Some(wood_shed) = state.wood_shed_state_mut() {
                        if wood_shed.axe_on_floor {
                            wood_shed.axe_on_floor = false;
                            picked_up = true;
                        }
                    }
                }
                if picked_up {
                    if state.player.inventory.add(Item::Axe, 1) {
                        state.on_player_pickup(&Item::Axe);
                        return InteractionResult::ItemObtained(
                            Item::Axe,
                            "You pick up the heavy axe.".to_string(),
                        );
                    } else {
                        if let Some(wood_shed) = state.wood_shed_state_mut() {
                            wood_shed.axe_on_floor = true;
                        }
                        return InteractionResult::Failure("Too heavy.".to_string());
                    }
                }
            }

            if item == Item::Log {
                let mut took_log = false;
                {
                    if let Some(wood_shed) = state.wood_shed_state_mut() {
                        if wood_shed.logs > 0 {
                            wood_shed.logs -= 1;
                            took_log = true;
                        }
                    }
                }
                if took_log {
                    if state.player.inventory.add(Item::Log, 1) {
                        state.on_player_pickup(&Item::Log);
                        let remaining = state.wood_shed_state().map(|w| w.logs).unwrap_or(0);
                        return InteractionResult::ItemObtained(
                            Item::Log,
                            format!("You heft a heavy log. {} remain.", remaining),
                        );
                    } else {
                        if let Some(wood_shed) = state.wood_shed_state_mut() {
                            wood_shed.logs += 1;
                        }
                        return InteractionResult::Failure("Carrying too much.".to_string());
                    }
                }
            }

            if item == Item::Firewood {
                let mut took_firewood = false;
                {
                    if let Some(wood_shed) = state.wood_shed_state_mut() {
                        if wood_shed.firewood > 0 {
                            wood_shed.firewood -= 1;
                            took_firewood = true;
                        }
                    }
                }
                if took_firewood {
                    if state.player.inventory.add(Item::Firewood, 1) {
                        state.on_player_pickup(&Item::Firewood);
                        return InteractionResult::ItemObtained(
                            Item::Firewood,
                            "You gather a piece of split firewood.".to_string(),
                        );
                    } else {
                        if let Some(wood_shed) = state.wood_shed_state_mut() {
                            wood_shed.firewood += 1;
                        }
                        return InteractionResult::Failure("Carrying too much.".to_string());
                    }
                }
            }
        }
        None => {
            // Outdoors - check tile items
            if let Some((r, c)) = state.player.position.as_usize() {
                if let Some(tile) = map.get_tile_mut(r, c) {
                    if tile.items.take(&item) {
                        if state.player.inventory.add(item.clone(), 1) {
                            state.on_player_pickup(&item);
                            return InteractionResult::ItemObtained(
                                item.clone(),
                                format!("You pick up the {}.", item.name()),
                            );
                        } else {
                            tile.items.add(item.clone(), 1); // Put it back
                            return InteractionResult::Failure(
                                "Your inventory is too heavy.".to_string(),
                            );
                        }
                    }
                }
            }
        }
        _ => {}
    }
    InteractionResult::Failure(format!(
        "You don't see any {} here that you can take.",
        item_name
    ))
}

pub fn try_drop(item_name: &str, state: &mut GameState, map: &mut WorldMap) -> InteractionResult {
    let item = match Item::from_str(item_name) {
        Some(i) => i,
        None => {
            return InteractionResult::Failure(format!("You don't know what '{}' is.", item_name))
        }
    };
    if !state.player.inventory.has(&item, 1) {
        return InteractionResult::Failure(format!("You don't have any {}.", item.name()));
    }
    state.player.inventory.remove(&item, 1);
    let dropped_book_id = state.on_player_drop(&item);
    match &state.player.room {
        Some(Room::CabinMain) => {
            if let Some(cabin) = state.cabin_state_mut() {
                cabin.add_item(item.clone());
            }
            if let Some(id) = dropped_book_id {
                state.add_cabin_book(id);
            }
        }
        Some(Room::WoodShed) => {
            if let Some(wood_shed) = state.wood_shed_state_mut() {
                match &item {
                    Item::Axe => wood_shed.axe_on_floor = true,
                    Item::Log => wood_shed.logs += 1,
                    Item::Firewood => wood_shed.firewood += 1,
                    _ => {}
                }
            }
        }
        None => {
            if let Some((r, c)) = state.player.position.as_usize() {
                if let Some(tile) = map.get_tile_mut(r, c) {
                    if item == Item::CardCase {
                        // Drop the card case itself on this tile
                        tile.items.add(Item::CardCase, 1);

                        // Scatter any cards currently inside the case around this tile
                        let cards_to_scatter = state.card_case_cards_inside.min(52);
                        state.card_case_cards_inside = 0;
                        state.card_case_open = false;

                        if cards_to_scatter > 0 {
                            let pos = state.player.position;
                            let mut rng = rand::thread_rng();
                            let mut positions = Vec::new();
                            for dr in -1..=1 {
                                for dc in -1..=1 {
                                    let p = Position::new(pos.row + dr, pos.col + dc);
                                    if let Some((rr, cc)) = p.as_usize() {
                                        positions.push((rr, cc));
                                    }
                                }
                            }
                            if !positions.is_empty() {
                                for _ in 0..cards_to_scatter {
                                    let &(rr, cc) =
                                        positions.get(rng.gen_range(0..positions.len())).unwrap();
                                    if let Some(t) = map.get_tile_mut(rr, cc) {
                                        t.items.add(Item::PlayingCard, 1);
                                    }
                                }
                            }
                        }

                    } else {
                        tile.items.add(item.clone(), 1);
                    }
                } else {
                    // Failed to place, return item
                    state.player.inventory.add(item.clone(), 1);
                    return InteractionResult::Failure(
                        "You fumble and fail to set that down here.".to_string(),
                    );
                }
            } else {
                state.player.inventory.add(item.clone(), 1);
                return InteractionResult::Failure(
                    "You fumble and fail to set that down here.".to_string(),
                );
            }
        }
        _ => {}
    }
    if item == Item::CardCase {
        let first_scatter = !state.card_scatter_achievement;
        if first_scatter {
            state.card_scatter_achievement = true;
        }
        let mut message =
            "You hurl the card case; cards explode into a chaotic ring around you."
                .to_string();
        if first_scatter {
            message.push(' ');
            message.push_str(
                "(Achievement unlocked: 52 Pickup. Sometimes you have to let things fly.)",
            );
        }
        InteractionResult::ItemLost(item.clone(), message)
    } else {
        InteractionResult::ItemLost(item.clone(), format!("You set down the {}.", item.name()))
    }
}

pub fn examine(target: &str, state: &GameState) -> String {
    let normalized = target.to_lowercase();
    let player = &state.player;

    // Check for active project
    if normalized.contains("blueprint") || normalized.contains("project") {
        if let Some(bp) = &player.active_project {
            return bp.status_description();
        } else {
            let mut parts = Vec::new();
            let known = state.known_blueprint_names();
            if !known.is_empty() {
                parts.push(format!("Known blueprints: {}", known.join(", ")));
            }
            let locked = state.locked_blueprint_hints();
            if !locked.is_empty() {
                parts.push(format!("Locked: {}", locked.join(" | ")));
            }
            let tail = if parts.is_empty() {
                String::new()
            } else {
                format!(" {}", parts.join(" "))
            };
            return format!(
                "You don't have any active blueprint. Use 'create [item]' to start one.{}",
                tail
            );
        }
    }

    if normalized.contains("book") || normalized.contains("note") || normalized.contains("책") {
        if let Some(book) = state.accessible_book(&normalized) {
            let page_info = if book.pages.is_empty() {
                "no pages yet".to_string()
            } else {
                format!("{} page(s)", book.pages.len())
            };
            return format!("Book [{}]: {} ({})", book.id, book.title, page_info);
        }
    }

    for (item, _) in state.player.inventory.list() {
        if item.name().to_lowercase().contains(&normalized) {
            return item.description().to_string();
        }
    }
    match &state.player.room {
        Some(Room::CabinMain) => {
            if normalized.contains("fire") || normalized.contains("hearth") {
                if let Some(cabin) = state.cabin_state() {
                    return cabin.fireplace.state.description().to_string();
                }
            }
            if normalized.contains("table") {
                let items = state.table_item_names();
                return if items.is_empty() {
                    "A sturdy wooden table, surface clear.".to_string()
                } else {
                    format!("A sturdy wooden table, holding: {}.", items.join(", "))
                };
            }
            // ... (other examine logic)
        }
        _ => {}
    }
    // ... (self examine)
    if normalized.contains("self") || normalized == "me" {
        return state.player.status_summary();
    }
    format!("You don't see anything special about '{}'.", target)
}

pub fn talk_to_rubber_duck(
    message: Option<&str>,
    state: &GameState,
    duck_name: &str,
) -> InteractionResult {
    let holding_duck = state.player.inventory.has(&Item::RubberDuck, 1);
    let duck_on_table = state
        .table_surface()
        .map(|s| s.items.contains(&Item::RubberDuck))
        .unwrap_or(false);
    let duck_in_cabin = state
        .cabin_state()
        .map(|c| c.items.contains(&Item::RubberDuck) || c.table_items.contains(&Item::RubberDuck))
        .unwrap_or(false);
    let in_cabin = matches!(state.player.room, Some(Room::CabinMain));

    if !(holding_duck || (in_cabin && (duck_in_cabin || duck_on_table))) {
        return InteractionResult::Failure("You need to be near the rubber duck.".to_string());
    }
    let mut rng = rand::thread_rng();
    let opener = match message {
        Some(msg) if !msg.trim().is_empty() => format!("You: \"{}\"\n", msg.trim()),
        _ => "You address the rubber duck softly.\n".to_string(),
    };
    let middle = "The rubber duck seems lost in thought...";
    let contemplation = random_duck_phrase(&mut rng);
    let closer = format!("{}: ...", duck_name);
    InteractionResult::Success(format!(
        "{}{}\n{}\n{}",
        opener, middle, contemplation, closer
    ))
}

// --- NEW UNIVERSAL USE HANDLER ---

pub fn try_use(
    item_name: &str,
    target_name: Option<&str>,
    state: &mut GameState,
    map: &mut WorldMap,
) -> InteractionResult {
    let item_query = item_name.trim();
    let item_query_lower = item_query.to_lowercase();
    let using_hands = matches!(
        item_query_lower.as_str(),
        "hand" | "hands" | "fist" | "fists" | "bare hands" | "bare-hands" | "barehand"
    );

    let target_normalized = target_name.as_ref().map(|s| s.to_lowercase());
    let target_str = target_normalized.as_deref();

    if using_hands {
        if let Some(target) = target_str {
            if target.contains("bush") || target.contains("shrub") || target.contains("ground") {
                return handle_foraging(state, None, map);
            }
        }
        return InteractionResult::Failure(
            "Use what with your hands? Try 'use hands on bush' to forage, or specify a tool and target."
                .to_string(),
        );
    }

    let item = match Item::from_str(item_query) {
        Some(i) => i,
        None => {
            return InteractionResult::Failure(format!("You don't know what '{}' is.", item_name))
        }
    };

    let mut has_item = state.player.inventory.has(&item, 1);
    if !has_item
        && matches!(
            item,
            Item::Book
                | Item::TutorialBook
                | Item::OldBook
                | Item::DeathNote
                | Item::BookOfFishing
                | Item::BlankBook
        )
        && matches!(state.player.room, Some(Room::CabinMain))
    {
        if let Some(cabin) = state.cabin_state() {
            let id_match = state
                .book_id_for_item(&item)
                .map(|bid| cabin.book_ids.iter().any(|b| b == bid))
                .unwrap_or(false);
            has_item = cabin.items.contains(&item) || cabin.table_items.contains(&item) || id_match;
        }
    }

    if !has_item {
        return InteractionResult::Failure(format!("You don't have a {}.", item.name()));
    }

    if item == Item::CardCase {
        return handle_card_case_use(state, map, target_str);
    }

    if item == Item::PlayingCard {
        if let Some(target) = target_str {
            if target.contains("case") || target.contains("card") {
                if !state.player.inventory.has(&Item::CardCase, 1) {
                    return InteractionResult::Failure(
                        "You need to be holding the card case to tuck this card away."
                            .to_string(),
                    );
                }
                if state.card_case_cards_inside >= 52 {
                    return InteractionResult::Failure(
                        "The card case is already full; you can't squeeze in another card."
                            .to_string(),
                    );
                }
                if !state.player.inventory.remove(&Item::PlayingCard, 1) {
                    return InteractionResult::Failure(
                        "You fumble and nearly drop the card; best try again more carefully."
                            .to_string(),
                    );
                }
                state.card_case_cards_inside =
                    (state.card_case_cards_inside.saturating_add(1)).min(52);
                return InteractionResult::Success(
                    "You slide the card neatly back into the case."
                        .to_string(),
                );
            }
        }
        return InteractionResult::Failure(
            "You flick the card between your fingers. Maybe use it with the card case?"
                .to_string(),
        );
    }

    if matches!(
        item,
        Item::Book | Item::TutorialBook | Item::OldBook | Item::DeathNote | Item::BookOfFishing
    ) {
        return handle_book_use(state, map, &item, target_str);
    }
    if item == Item::BlankBook {
        return InteractionResult::Failure(
            "It's a blank book. Title it first with 'write 제목:<title> on 빈 책'.".to_string(),
        );
    }

    // 1. Blueprint Interaction (Building)
    let target_is_blueprint = target_str
        .map(|t| t.contains("blueprint") || t.contains("project"))
        .unwrap_or(false);
    if target_is_blueprint {
        return handle_blueprint_interaction(state, &item);
    }
    // Also check if target is the name of the blueprint item or if no target is given but material matches
    if let Some(bp) = &state.player.active_project {
        if target_str
            .map(|t| bp.target_item.name().to_lowercase().contains(t))
            .unwrap_or(false)
            || (target_str.is_none() && bp.required.contains_key(&item))
        {
            return handle_blueprint_interaction(state, &item);
        }
    }

    // 2. Resource Gathering (Chopping, etc)
    if let Some(target) = target_str {
        if target.contains("bamboo") {
            if item == Item::Axe || item == Item::StoneAxe {
                return try_chop_tree(state, map, &item);
            }
        }
        if target.contains("tree") || target.contains("wood") || target.contains("log") {
            if item == Item::Axe || item == Item::StoneAxe {
                // Check if it's chopping block or standing tree
                if target.contains("block") || target.contains("chop") {
                    return try_chop_firewood(state, &item);
                } else {
                    return try_chop_tree(state, map, &item);
                }
            }
        }
        if target.contains("bush") || target.contains("shrub") || target.contains("ground") {
            return handle_foraging(state, Some(&item), map);
        }
    }

    // 3. Processing (Crafting Materials)
    if item == Item::Knife || item == Item::StoneKnife {
        if let Some(target) = target_str {
            if target.contains("log") {
                if state.player.inventory.has(&Item::Log, 1) {
                    state.player.inventory.remove(&Item::Log, 1);
                    state.player.inventory.add(Item::Kindling, 4);
                    state.player.skills.improve("woodcutting", 2);
                    state.damage_tool(&item, 1, "whittling wood");
                    return InteractionResult::ActionSuccess {
                        message: "You whittle the log down into a pile of fine kindling."
                            .to_string(),
                        time_cost: 2, // 20 mins
                        energy_cost: 10.0,
                    };
                }
            }
            if target.contains("branch") || target.contains("stick") {
                if state.player.inventory.has(&Item::Stick, 1) {
                    state.player.inventory.remove(&Item::Stick, 1);
                    state.player.inventory.add(Item::Kindling, 1);
                    state.player.skills.improve("woodcutting", 1);
                    state.damage_tool(&item, 1, "carving a stick");
                    return InteractionResult::ActionSuccess {
                        message: "You shave the stick into tinder.".to_string(),
                        time_cost: 1,
                        energy_cost: 2.0,
                    };
                }
            }
            if target.contains("bamboo") {
                if state.player.inventory.has(&Item::Bamboo, 1) {
                    state.player.inventory.remove(&Item::Bamboo, 1);
                    state.player.inventory.add(Item::Paper, 3);
                    state.player.skills.improve("tailoring", 2);
                    state.damage_tool(&item, 1, "splitting bamboo");
                    return InteractionResult::ActionSuccess {
                        message: "You split the bamboo and press it into thin sheets of paper."
                            .to_string(),
                        time_cost: 2,
                        energy_cost: 6.0,
                    };
                } else {
                    return InteractionResult::Failure(
                        "You need bamboo in your inventory to cut into paper.".to_string(),
                    );
                }
            }
        }
    }

    // Raft: short lake excursion for observations
    if item == Item::Raft {
        let pos = state.player.position;
        let mut near_water = false;
        'outer: for dr in -1..=1 {
            for dc in -1..=1 {
                let check = Position::new(pos.row + dr, pos.col + dc);
                if let Some((r, c)) = check.as_usize() {
                    if let Some(tile) = map.get_tile(r, c) {
                        if matches!(tile.biome, Biome::Lake | Biome::Oasis) {
                            near_water = true;
                            break 'outer;
                        }
                    }
                }
            }
        }
        if !near_water {
            return InteractionResult::Failure(
                "Find a shoreline first; you need water to launch the raft.".to_string(),
            );
        }

        let weather_here = state.weather.get_for_position(pos.row, pos.col);
        let severe = matches!(
            weather_here,
            Weather::Blizzard | Weather::HeavySnow | Weather::HeavyRain | Weather::Sandstorm
        );
        let mut time_cost = 3;
        let mut energy_cost = 8.0;
        if severe {
            time_cost += 1;
            energy_cost += 2.0;
        }

        let mut rng = rand::thread_rng();
        let mut findings = Vec::new();
        if rng.gen_bool(0.5) {
            findings.push("a glint of fish beneath the surface");
            state.player.skills.improve("observation", 2);
        } else {
            state.player.skills.improve("observation", 1);
        }
        if rng.gen_bool(0.35) {
            if state.player.inventory.add(Item::Driftwood, 1) {
                findings.push("a floating piece of driftwood you haul aboard");
            }
        }

        state.player.modify_mood(3.0);
        let finding_text = if findings.is_empty() {
            "You mostly drift and listen to the water slap the hull.".to_string()
        } else {
            findings.join("; ")
        };

        return InteractionResult::ActionSuccess {
            message: format!(
                "You slide the raft into the water and paddle out, letting the cabin shrink behind you. {}",
                finding_text
            ),
            time_cost,
            energy_cost,
        };
    }

    // 3b. Cooking simple foods on fire
    if matches!(
        item,
        Item::Fish | Item::SmallFish | Item::BigFish | Item::WildBerry
    ) {
        let in_cabin = matches!(state.player.room, Some(Room::CabinMain));
        let fire_lit = state
            .cabin_state()
            .map(|c| !matches!(c.fireplace.state, FireState::Cold))
            .unwrap_or(false);
        if !in_cabin || !fire_lit {
            return InteractionResult::Failure(
                "You need to be by a lit fireplace to cook that right now.".to_string(),
            );
        }

        let severe = {
            let pos = state.player.position;
            let weather_here = state.weather.get_for_position(pos.row, pos.col);
            matches!(
                weather_here,
                Weather::Blizzard | Weather::HeavySnow | Weather::HeavyRain | Weather::Sandstorm
            )
        };
        let mut time_cost = 2;
        let mut energy_cost = 4.0;
        if severe {
            time_cost += 1;
            energy_cost += 2.0;
        }

        if matches!(item, Item::Fish | Item::SmallFish | Item::BigFish) {
            if !state.player.inventory.remove(&item, 1) {
                return InteractionResult::Failure("You don't have a fish to cook.".to_string());
            }
            let yield_count = if item == Item::BigFish { 2 } else { 1 };
            if item == Item::BigFish {
                time_cost += 1;
            }
            state.player.inventory.add(Item::CookedFish, yield_count);
            let portion_text = if yield_count > 1 {
                "You portion the large fish into hearty fillets and grill them until they flake easily."
            } else {
                "You grill the fish over the fire until it flakes easily."
            };
            return InteractionResult::ActionSuccess {
                message: portion_text.to_string(),
                time_cost,
                energy_cost,
            };
        } else {
            if state.player.inventory.count(&Item::WildBerry) < 2 {
                return InteractionResult::Failure(
                    "Gather at least a couple of berries to roast.".to_string(),
                );
            }
            state.player.inventory.remove(&Item::WildBerry, 2);
            state.player.inventory.add(Item::CookedBerries, 1);
            return InteractionResult::ActionSuccess {
                message: "You roast the berries, caramelizing their juices.".to_string(),
                time_cost,
                energy_cost,
            };
        }
    }

    // Stone on Stone -> Sharp Stone
    if item == Item::Stone {
        if let Some(target) = target_str {
            if target.contains("stone") || target.contains("rock") {
                if state.player.inventory.count(&Item::Stone) >= 2 {
                    state.player.inventory.remove(&Item::Stone, 1);
                    state.player.inventory.add(Item::SharpStone, 1);
                    state.player.skills.improve("stonemasonry", 5);
                    return InteractionResult::ActionSuccess {
                        message: "You smash the stones together, flaking off a razor-sharp edge."
                            .to_string(),
                        time_cost: 1,
                        energy_cost: 5.0,
                    };
                } else {
                    return InteractionResult::Failure(
                        "You need another stone to knap against.".to_string(),
                    );
                }
            }
        }
    }

    // Paper on paper -> Blank Book
    if item == Item::Paper {
        if state.player.inventory.count(&Item::Paper) >= 5 {
            state.player.inventory.remove(&Item::Paper, 5);
            state.player.inventory.add(Item::BlankBook, 1);
            state.player.skills.improve("tailoring", 1);
            return InteractionResult::ActionSuccess {
                message: "You fold and bind the sheets into a blank book. Try 'write 제목:... on 빈 책' to title it.".to_string(),
                time_cost: 2,
                energy_cost: 3.0,
            };
        } else {
            return InteractionResult::Failure(
                "You need at least 5 sheets of paper to bind a blank book.".to_string(),
            );
        }
    }

    // 4. Fire Interaction
    let is_fire_target = target_str
        .map(|t| t.contains("fire") || t.contains("hearth"))
        .unwrap_or(false);
    let in_cabin = matches!(state.player.room, Some(Room::CabinMain));

    if is_fire_target || (in_cabin && target_str.is_none()) {
        if item.is_flammable() {
            return handle_add_fuel(state, item);
        }
        if item == Item::Matchbox {
            return handle_light_fire(state);
        }
    }

    // 5. Consumption (Food/Drink)
    if matches!(
        item,
        Item::Apple
            | Item::WildBerry
            | Item::HerbalTea
            | Item::MuddyWater
            | Item::CleanWater
            | Item::Date
            | Item::CookedFish
            | Item::CookedBerries
            | Item::SmallFish
            | Item::Fish
            | Item::BigFish
    ) {
        return handle_consumption(state, item);
    }

    InteractionResult::Failure(format!(
        "You can't use the {} that way. Try patterns like: use axe on tree (gather), use knife on stick (process), or use log on blueprint (build).",
        item.name()
    ))
}

fn handle_card_case_use(
    state: &mut GameState,
    map: &mut WorldMap,
    target: Option<&str>,
) -> InteractionResult {
    let pos = state.player.position;
    let Some((r, c)) = pos.as_usize() else {
        return InteractionResult::Failure(
            "You feel strangely ungrounded; the card case slips in your hands.".to_string(),
        );
    };

    let Some(tile) = map.get_tile_mut(r, c) else {
        return InteractionResult::Failure(
            "You can't quite find space here to lay out cards.".to_string(),
        );
    };

    let cards_on_ground = tile
        .items
        .items
        .iter()
        .find_map(|(item, qty)| {
            if *item == Item::PlayingCard && *qty > 0 {
                Some(*qty)
            } else {
                None
            }
        })
        .unwrap_or(0);

    let target_hint = target.unwrap_or("").to_lowercase();

    // If the case is closed, using it mostly acts like flipping it open.
    if !state.card_case_open && state.card_case_cards_inside > 0 && cards_on_ground == 0 {
        state.card_case_open = true;
        return InteractionResult::Success(
            "You flick open the card case. A full deck rests inside, edges whispering together."
                .to_string(),
        );
    }

    // Open case with cards inside and clear ground -> lay them out
    if state.card_case_open && state.card_case_cards_inside > 0 && cards_on_ground == 0 {
        let inside = state.card_case_cards_inside.min(52);
        state.card_case_cards_inside = state.card_case_cards_inside.saturating_sub(inside);
        tile.items.add(Item::PlayingCard, inside as u32);
        return InteractionResult::Success(format!(
            "You slide {} cards out of the case and spread them across the ground.",
            inside
        ));
    }

    // Open case, cards on ground -> scoop back in up to full deck
    if state.card_case_open && cards_on_ground > 0 {
        let capacity_left = 52u8.saturating_sub(state.card_case_cards_inside);
        if capacity_left == 0 {
            return InteractionResult::Failure(
                "The card case is already holding a full deck.".to_string(),
            );
        }

        let mut moved: u8 = 0;
        while moved < capacity_left && tile.items.take(&Item::PlayingCard) {
            moved = moved.saturating_add(1);
        }

        if moved == 0 {
            return InteractionResult::Failure(
                "You don't see any cards here to scoop into the case.".to_string(),
            );
        }

        state.card_case_cards_inside =
            (state.card_case_cards_inside.saturating_add(moved)).min(52);

        return InteractionResult::Success(format!(
            "You gather {} cards from the ground back into the case.",
            moved
        ));
    }

    // Closed or empty case with no cards nearby
    if !state.card_case_open && state.card_case_cards_inside == 0 && cards_on_ground == 0 {
        return InteractionResult::Failure(
            "The card case feels light and empty. There are no cards here to work with."
                .to_string(),
        );
    }

    // Fallback: if there are cards on the ground but the case is closed, hint to open it first
    if !state.card_case_open && cards_on_ground > 0 {
        let verb = if target_hint.contains("open") {
            "Use 'open card case' first, then use the case again to scoop cards from the ground."
        } else {
            "Open the card case first, then use it again to scoop cards from the ground."
        };
        return InteractionResult::Failure(verb.to_string());
    }

    InteractionResult::Failure(
        "You turn the card case over in your hands, but nothing interesting happens."
            .to_string(),
    )
}

fn parse_book_id_from_target(target: Option<&str>) -> Option<String> {
    let target = target?.trim();
    if target.is_empty() {
        return None;
    }
    if let Some(start) = target.find('(') {
        if let Some(end_rel) = target[start + 1..].find(')') {
            return Some(target[start + 1..start + 1 + end_rel].trim().to_string());
        }
    }
    Some(target.to_string())
}

fn handle_book_use(
    state: &mut GameState,
    map: &mut WorldMap,
    item: &Item,
    target: Option<&str>,
) -> InteractionResult {
    let id_from_item = state.book_id_for_item(item).map(|s| s.to_string());
    let mut accessible_ids = state.accessible_book_ids();
    let book_id = if let Some(id) = id_from_item {
        id
    } else if let Some(id) = parse_book_id_from_target(target) {
        id
    } else if accessible_ids.len() == 1 {
        accessible_ids.pop().unwrap()
    } else {
        accessible_ids.sort();
        let listing = if accessible_ids.is_empty() {
            "No book IDs available. Bind a blank book first with 'write 제목:<title> on 빈 책'."
                .to_string()
        } else {
            format!(
                "Specify which book to read. Available: {}",
                accessible_ids.join(", ")
            )
        };
        return InteractionResult::Failure(listing);
    };

    if !state.player_or_cabin_has_book(&book_id) {
        return InteractionResult::Failure(
            "You need to hold that book (or be next to it in the cabin).".to_string(),
        );
    }

    let Some(book) = state.books.get(&book_id) else {
        return InteractionResult::Failure("That book doesn't seem to exist.".to_string());
    };
    let title = book.title.clone();
    let total_pages = book.pages.len();
    let pages_copy = book.pages.clone();
    let book_label = book.id.clone();

    let mut page = state.book_page(&book_id);
    if let Some(t) = target {
        let lower = t.to_lowercase();
        if lower.contains("next") {
            page = page.saturating_add(1);
        } else if lower.contains("prev") || lower.contains("previous") {
            page = page.saturating_sub(1);
        }
    }

    let max_page = book.pages.len();
    if page > max_page {
        page = max_page;
    }
    state.set_book_page(&book_id, page);
    state.refresh_blueprint_knowledge(true);
    state.grant_tutorial_reward_if_needed(map);

    let message = if page == 0 {
        format!(
            "{} [{}] — cover page. Total pages: {}. Use 'use {} on nextpage' to turn pages.",
            title,
            book_label,
            total_pages,
            item.name()
        )
    } else {
        let content = pages_copy
            .get(page - 1)
            .map(|s| s.as_str())
            .unwrap_or("This page is blank.");
        format!("{} [{}] — Page {}: {}", title, book_label, page, content)
    };

    InteractionResult::Success(message)
}

fn handle_blueprint_interaction(state: &mut GameState, item: &Item) -> InteractionResult {
    if let Some(bp) = &mut state.player.active_project {
        if bp.add_material(item.clone()) {
            state.player.inventory.remove(item, 1);
            if !bp.is_complete() {
                let progress = bp.progress_summary();
                return InteractionResult::ActionSuccess {
                    message: format!(
                        "You add the {} to the {}. Progress: {}. Total build time: {} mins.",
                        item.name(),
                        bp.target_item.name(),
                        progress,
                        bp.time_cost
                    ),
                    time_cost: 1, // 10 mins per action
                    energy_cost: 2.0,
                };
            }
        } else {
            return InteractionResult::Failure(format!(
                "The {} doesn't need any (more) {}.",
                bp.target_item.name(),
                item.name()
            ));
        }
    } else {
        return InteractionResult::Failure(
            "You don't have an active blueprint. Use 'create [item]' first.".to_string(),
        );
    }

    if let Some(bp) = state.player.active_project.take() {
        state.player.inventory.add(bp.target_item.clone(), 1);

        // Skill gain based on item type
        match bp.target_item {
            Item::StoneKnife | Item::StoneAxe => state.player.skills.improve("stonemasonry", 10),
            Item::Campfire => state.player.skills.improve("survival", 5),
            Item::Cordage => state.player.skills.improve("tailoring", 5),
            _ => {}
        }

        let time_cost = ((bp.time_cost + 9) / 10).max(1);
        let energy_cost = (time_cost as f32 * 2.0).max(5.0);

        return InteractionResult::ActionSuccess {
            message: format!(
                "You finish crafting the {}. It is ready to use.",
                bp.target_item.name()
            ),
            time_cost,
            energy_cost,
        };
    }

    InteractionResult::Failure("Something went wrong with the blueprint.".to_string())
}

fn handle_foraging(
    state: &mut GameState,
    tool: Option<&Item>,
    map: &WorldMap,
) -> InteractionResult {
    let mut rng = rand::thread_rng();
    let skill = state.player.skills.get("foraging");
    let pos = state.player.position;

    // Check energy
    if state.player.energy < 5.0 {
        return InteractionResult::Failure("You are too exhausted to forage.".to_string());
    }

    let tool_bonus = matches!(
        tool,
        Some(Item::Knife | Item::StoneKnife | Item::Axe | Item::StoneAxe)
    );
    let success_chance =
        (0.6 + (skill as f64 * 0.005) + if tool_bonus { 0.1 } else { 0.0 }).min(0.95);

    // Local biome can tilt what we find
    let biome = pos
        .as_usize()
        .and_then(|(r, c)| map.get_tile(r, c).map(|t| t.biome))
        .unwrap_or(Biome::MixedForest);

    // Forage node depletion
    state.foraging_node_for(pos, map, &mut rng);
    let depleted = state
        .forage_nodes
        .get(&pos)
        .map(|n| n.charges == 0)
        .unwrap_or(false);
    if depleted {
        return InteractionResult::Failure(
            "The brush here is picked clean. Give it some time to recover.".to_string(),
        );
    }

    // Drops
    let drops = if rng.gen_bool(success_chance) {
        // Success: always some basic materials, with better food odds in lush biomes

        // Sticks: base 1, sometimes more as skill improves
        let mut stick_count = 1;
        if rng.gen_bool((0.3 + skill as f64 * 0.01).min(0.8)) {
            stick_count += 1;
        }
        state.player.inventory.add(Item::Stick, stick_count);

        // Plant fiber: more common with tools and skill
        let fiber_chance = (0.35 + skill as f64 * 0.005 + if tool_bonus { 0.15 } else { 0.0 })
            .min(0.85);
        let fiber_rolls = if tool_bonus { 2 } else { 1 };
        for _ in 0..fiber_rolls {
            if rng.gen_bool(fiber_chance) {
                state.player.inventory.add(Item::PlantFiber, 1);
            }
        }

        // Stone: slightly more likely than before
        if rng.gen_bool(0.3) {
            state.player.inventory.add(Item::PlantFiber, 1);
        }
        if rng.gen_bool(0.25) {
            state.player.inventory.add(Item::Stone, 1);
        }

        // Food-focused drops
        let mut food_found = 0u32;

        // Berries: more generous in forest/oasis-style biomes
        let berry_base = 0.25;
        let berry_biome_bonus = match biome {
            Biome::SpringForest | Biome::MixedForest | Biome::BambooGrove | Biome::Oasis => 0.25,
            Biome::Lake => 0.15,
            _ => 0.0,
        };
        let berry_chance =
            (berry_base + berry_biome_bonus + (skill as f64 * 0.005)).min(0.9);
        let mut berry_count = 0u32;
        for _ in 0..3 {
            if rng.gen_bool(berry_chance) {
                berry_count += 1;
            }
        }
        if berry_count > 0 {
            state
                .player
                .inventory
                .add(Item::WildBerry, berry_count as u32);
            food_found += berry_count;
        }

        // Dates in oasis/desert edge
        if matches!(biome, Biome::Oasis | Biome::Desert) && rng.gen_bool(0.15) {
            state.player.inventory.add(Item::Date, 1);
            food_found += 1;
        }

        // Occasional edible herbs for tea
        if rng.gen_bool(0.12) {
            state.player.inventory.add(Item::WildHerbs, 1);
            food_found += 1;
        }

        state.player.skills.improve("foraging", 1);
        if let Some(node) = state.forage_nodes.get_mut(&pos) {
            node.charges = node.charges.saturating_sub(1);
            if node.charges == 0 {
                node.cooldown = 12;
            }
        }

        if let Some(t) = tool {
            state.damage_tool(t, 1, "foraging");
        }

        InteractionResult::ActionSuccess {
            message: if food_found > 0 {
                "You rummage through the bushes and come away with something to eat and a handful of useful materials."
                    .to_string()
            } else {
                "You rummage through the brush and find useful materials.".to_string()
            },
            time_cost: 1, // 10 mins
            energy_cost: 5.0,
        }
    } else {
        InteractionResult::ActionSuccess {
            message: "You search for a while but find nothing useful.".to_string(),
            time_cost: 1,
            energy_cost: 3.0,
        }
    };
    drops
}

fn try_chop_firewood(state: &mut GameState, tool: &Item) -> InteractionResult {
    if !matches!(state.player.room, Some(Room::WoodShed)) {
        return InteractionResult::Failure("Go to the wood shed to chop firewood.".to_string());
    }
    // ... (Simplified logic for brevity, using ActionSuccess)
    if let Some(wood_shed) = state.wood_shed_state_mut() {
        if wood_shed.logs > 0 {
            wood_shed.logs -= 1;
            state.player.inventory.add(Item::Firewood, 3);
            state.player.skills.improve("woodcutting", 2);
            let result = InteractionResult::ActionSuccess {
                message: "You chop a log into firewood.".to_string(),
                time_cost: 2,
                energy_cost: 10.0,
            };
            state.damage_tool(tool, 2, "splitting firewood");
            result
        } else {
            InteractionResult::Failure("No logs in the shed.".to_string())
        }
    } else {
        InteractionResult::Failure("The wood shed isn't available right now.".to_string())
    }
}

// Re-implement tree chopping with ActionSuccess
fn try_chop_tree(state: &mut GameState, _map: &WorldMap, tool: &Item) -> InteractionResult {
    let player_pos = state.player.position;
    let Some(tree) = state.objects.find_tree_mut_at(&player_pos) else {
        return InteractionResult::Failure(
            "There isn't a standing tree right here to chop.".to_string(),
        );
    };
    if tree.felled {
        return InteractionResult::Failure("This tree has already been felled.".to_string());
    }

    if matches!(tree.kind, crate::entity::TreeType::Bamboo) {
        tree.felled = true;
        state.player.inventory.add(Item::Bamboo, 2);
        state.player.skills.improve("woodcutting", 3);
        let result = InteractionResult::ActionSuccess {
            message: "You slice through the bamboo. The stalks fall neatly.".to_string(),
            time_cost: 2,
            energy_cost: 10.0,
        };
        state.damage_tool(tool, 1, "cutting bamboo");
        return result;
    }

    tree.felled = true;
    state.player.inventory.add(Item::Log, 2);
    state.player.inventory.add(Item::Kindling, 1);
    state.player.inventory.add(Item::Bark, 1);
    state.player.skills.improve("woodcutting", 5);

    let result = InteractionResult::ActionSuccess {
        message: "You fell a tree! Timber!".to_string(),
        time_cost: 6, // 1 hour
        energy_cost: 20.0,
    };
    state.damage_tool(tool, 3, "chopping a tree");
    result
}

fn handle_add_fuel(state: &mut GameState, item: Item) -> InteractionResult {
    state.player.inventory.remove(&item, 1);
    if let Some(cabin) = state.cabin_state_mut() {
        if cabin.fireplace.add_fuel_item(item) {
            state.player.skills.improve("fire_making", 1);
            let time_cost = if matches!(item, Item::Log | Item::Firewood) {
                2
            } else {
                1
            };
            let energy_cost = if matches!(item, Item::Log | Item::Firewood) {
                3.0
            } else {
                1.0
            };
            return InteractionResult::ActionSuccess {
                message: format!("You add {} to the fire.", item.name()),
                time_cost,
                energy_cost,
            };
        }
    }
    state.player.inventory.add(item, 1);
    InteractionResult::Failure("It won't burn.".to_string())
}

fn handle_light_fire(state: &mut GameState) -> InteractionResult {
    if let Some(cabin) = state.cabin_state_mut() {
        if cabin.fireplace.ignite() {
            state.player.skills.improve("fire_making", 2);
            return InteractionResult::ActionSuccess {
                message: "You strike a match and the fire catches!".to_string(),
                time_cost: 1,
                energy_cost: 1.0,
            };
        } else {
            return InteractionResult::Failure(
                "You need tinder and fuel to start a fire.".to_string(),
            );
        }
    }
    InteractionResult::Failure("There's no hearth here.".to_string())
}

fn handle_consumption(state: &mut GameState, item: Item) -> InteractionResult {
    state.player.inventory.remove(&item, 1);
    let message = match item {
        Item::Apple => {
            state.player.modify_fullness(15.0);
            "You eat the apple.".to_string()
        }
        Item::WildBerry => {
            state.player.modify_fullness(5.0);
            state.player.modify_mood(2.0);
            "You snack on the berries.".to_string()
        }
        Item::Date => {
            state.player.modify_fullness(10.0);
            state.player.modify_hydration(8.0);
            state.player.modify_mood(2.0);
            "Sweet dates revive you with a burst of sugar and moisture.".to_string()
        }
        Item::CleanWater => {
            state.player.modify_hydration(25.0);
            state.player.modify_energy(2.0);
            "You drink the clean water. It tastes refreshing.".to_string()
        }
        Item::MuddyWater => {
            state.player.modify_hydration(8.0);
            state.player.modify_health(-4.0);
            state.player.modify_mood(-3.0);
            "You choke down the muddy water. It sits poorly in your stomach.".to_string()
        }
        Item::SmallFish | Item::Fish => {
            state.player.modify_fullness(14.0);
            state.player.modify_health(-1.0);
            state.player.modify_mood(-2.0);
            "You swallow the raw fish. It's briny and not entirely pleasant.".to_string()
        }
        Item::BigFish => {
            state.player.modify_fullness(22.0);
            state.player.modify_health(-2.0);
            state.player.modify_mood(-3.0);
            "You eat chunks of raw fish. It fills you, though it sits heavy.".to_string()
        }
        Item::CookedFish => {
            state.player.modify_fullness(30.0);
            state.player.modify_mood(4.0);
            "You eat the warm, cooked fish. Protein and warmth spread through you.".to_string()
        }
        Item::CookedBerries => {
            state.player.modify_fullness(12.0);
            state.player.modify_mood(6.0);
            "You munch on the roasted berries. Sweet and tart.".to_string()
        }
        Item::HerbalTea => {
            state.player.modify_hydration(15.0);
            state.player.modify_mood(5.0);
            state.player.modify_warmth(3.0);
            "You sip the herbal tea, feeling calm and warm.".to_string()
        }
        _ => format!("You consume the {}.", item.name()),
    };

    InteractionResult::ActionSuccess {
        message,
        time_cost: 1,
        energy_cost: 0.0,
    }
}

pub fn try_fish(
    state: &mut GameState,
    map: &WorldMap,
    gear_hint: Option<&str>,
) -> InteractionResult {
    let pos = state.player.position;
    let mut near_water = false;

    'outer: for dr in -1..=1 {
        for dc in -1..=1 {
            let check = Position::new(pos.row + dr, pos.col + dc);
            if let Some((r, c)) = check.as_usize() {
                if let Some(tile) = map.get_tile(r, c) {
                    if matches!(tile.biome, Biome::Lake | Biome::Oasis) {
                        near_water = true;
                        break 'outer;
                    }
                }
            }
        }
    }

    if !near_water {
        return InteractionResult::Failure(
            "You need to be right by the lake or oasis shore to fish.".to_string(),
        );
    }

    if state.player.energy < 5.0 {
        return InteractionResult::Failure("You are too exhausted to fish right now.".to_string());
    }

    let has_rod = state.player.inventory.has(&Item::FishingRod, 1);
    let wants_rod = gear_hint
        .map(|g| {
            let lower = g.to_lowercase();
            lower.contains("rod") || lower.contains("pole")
        })
        .unwrap_or(has_rod);
    if wants_rod && !has_rod {
        return InteractionResult::Failure(
            "You reach for a rod, but you don't have one with you.".to_string(),
        );
    }
    let using_rod = wants_rod && has_rod;

    let weather_here = state.weather.get_for_position(pos.row, pos.col);
    let tod = state.time.time_of_day();
    let stormy = matches!(
        weather_here,
        Weather::HeavyRain | Weather::HeavySnow | Weather::Blizzard | Weather::Sandstorm
    );

    let mut outcomes: Vec<(&str, u32)> = if using_rod {
        vec![("small", 45), ("big", 18), ("trash", 12), ("nothing", 25)]
    } else {
        vec![("small", 25), ("big", 6), ("trash", 20), ("nothing", 49)]
    };

    if matches!(tod, TimeOfDay::Dawn | TimeOfDay::Dusk | TimeOfDay::Evening) {
        outcomes[0].1 += 6;
        outcomes[1].1 += 4;
        outcomes[3].1 = outcomes[3].1.saturating_sub(8);
    }

    if stormy {
        outcomes[0].1 = outcomes[0].1.saturating_sub(5);
        outcomes[1].1 = outcomes[1].1.saturating_sub(3);
        outcomes[2].1 += 6;
        outcomes[3].1 += 6;
    }

    let skill_bonus = (state.player.skills.get("survival") as u32 / 12)
        + (state.player.skills.get("observation") as u32 / 20);
    if skill_bonus > 0 {
        outcomes[0].1 += skill_bonus;
        outcomes[3].1 = outcomes[3].1.saturating_sub(skill_bonus.min(outcomes[3].1));
    }

    let total: u32 = outcomes.iter().map(|(_, w)| *w).sum::<u32>().max(1);
    let roll = rand::thread_rng().gen_range(0..total);
    let mut cursor = 0;
    let chosen = outcomes
        .iter()
        .find(|(_, weight)| {
            cursor += *weight;
            roll < cursor
        })
        .map(|(name, _)| *name)
        .unwrap_or("nothing");

    let mut time_cost = if using_rod { 2 } else { 1 };
    let mut energy_cost = if using_rod { 5.0 } else { 4.0 };
    if stormy {
        time_cost += 1;
        energy_cost += 2.0;
    }

    let message = match chosen {
        "small" => {
            if !state.player.inventory.add(Item::SmallFish, 1) {
                return InteractionResult::Failure(
                    "Your pack is too heavy to stow the fish.".to_string(),
                );
            }
            state.player.skills.improve("survival", 2);
            state.player.skills.improve("observation", 1);
            "You feel a quick tug and pull up a small fish, cool and slick in your hand."
                .to_string()
        }
        "big" => {
            if !state.player.inventory.add(Item::BigFish, 1) {
                return InteractionResult::Failure(
                    "The catch is too heavy for your current pack.".to_string(),
                );
            }
            state.player.skills.improve("survival", 3);
            state.player.skills.improve("observation", 1);
            time_cost += 1;
            energy_cost += 1.0;
            "A strong pull bends your line. After a short struggle you haul in a hefty fish."
                .to_string()
        }
        "trash" => {
            if !state.player.inventory.add(Item::Driftwood, 1) {
                return InteractionResult::Failure(
                    "You snag some driftwood, but you're carrying too much to keep it.".to_string(),
                );
            }
            state.player.skills.improve("survival", 1);
            "Your line goes taut on something lifeless. You drag in a piece of driftwood."
                .to_string()
        }
        _ => {
            state.player.skills.improve("survival", 1);
            "You wait with quiet patience, but nothing bites this time.".to_string()
        }
    };

    if using_rod {
        state.damage_tool(&Item::FishingRod, 1, "casting for fish");
    }

    InteractionResult::ActionSuccess {
        message,
        time_cost,
        energy_cost,
    }
}

// New Create command handler
pub fn try_create(item_name: &str, state: &mut GameState) -> InteractionResult {
    let target_item = match Item::from_str(item_name) {
        Some(i) => i,
        None => return InteractionResult::Failure(format!("Unknown item '{}'.", item_name)),
    };

    let recipe_available = Blueprint::new(target_item).is_some();
    if !recipe_available {
        return InteractionResult::Failure(format!("You don't know how to craft a {}.", item_name));
    }

    state.refresh_blueprint_knowledge(true);

    if !state.knows_blueprint(target_item) {
        let mut msg = format!(
            "You haven't learned the {} blueprint yet.",
            target_item.name()
        );
        if let Some(hint) = state.blueprint_hint_text(target_item) {
            msg.push(' ');
            msg.push_str(hint);
        }
        let known = state.known_blueprint_names();
        if !known.is_empty() {
            msg.push_str(&format!(" Known blueprints: {}.", known.join(", ")));
        }
        return InteractionResult::Failure(msg);
    }

    let bp = Blueprint::new(target_item).unwrap();
    let progress = bp.progress_summary();
    let time_cost = bp.time_cost;
    state.player.active_project = Some(bp);
    InteractionResult::Success(format!(
        "You lay out plans for a {}. Requires: {}. Total build time: {} mins.",
        target_item.name(),
        progress,
        time_cost
    ))
}

pub fn write_on_book(text: &str, target: &str, state: &mut GameState) -> InteractionResult {
    let content = text.trim();
    if content.is_empty() {
        return InteractionResult::Failure("Provide text to write, e.g., 'write 제목:My Book on 빈 책' or 'write 페이지1:Hello on book-3'.".to_string());
    }

    let lower = content.to_lowercase();
    let is_title = lower.starts_with("제목:") || lower.starts_with("title:");
    let is_page = lower.starts_with("페이지") || lower.starts_with("page");

    if is_title {
        let title = content
            .split_once(':')
            .map(|(_, t)| t.trim())
            .unwrap_or("")
            .to_string();
        if title.is_empty() {
            return InteractionResult::Failure(
                "Please provide a title after '제목:' or 'title:'.".to_string(),
            );
        }
        if !state.player.inventory.has(&Item::BlankBook, 1) {
            return InteractionResult::Failure(
                "You need a blank book to bind a title.".to_string(),
            );
        }
        state.player.inventory.remove(&Item::BlankBook, 1);
        state.player.inventory.add(Item::Book, 1);
        let id = state.generate_book_id();
        let entry = BookEntry::new(id.clone(), title, true);
        state.register_book(entry);
        state.add_player_book(&id);
        return InteractionResult::ActionSuccess {
            message: format!("You title the book and bind it. Book ID: {}.", id),
            time_cost: 1,
            energy_cost: 1.0,
        };
    }

    if !is_page {
        return InteractionResult::Failure("Unsupported write format. Use '제목:<title>' for blank books or '페이지<number>:<text>' for existing books.".to_string());
    }

    let (page_spec, body) = match content.split_once(':') {
        Some(parts) => parts,
        None => {
            return InteractionResult::Failure(
                "Use '페이지<number>:<text>' to write a page.".to_string(),
            )
        }
    };

    let digits: String = page_spec.chars().filter(|c| c.is_ascii_digit()).collect();
    let page_num: usize = digits.parse().unwrap_or(0);
    if page_num == 0 {
        return InteractionResult::Failure(
            "Specify a page number like 페이지1 or page2.".to_string(),
        );
    }

    let book_id = {
        if let Some(start) = target.find('(') {
            if let Some(end) = target[start + 1..].find(')') {
                target[start + 1..start + 1 + end].trim().to_string()
            } else {
                target.trim().to_string()
            }
        } else {
            target.trim().to_string()
        }
    };

    if book_id.is_empty() {
        return InteractionResult::Failure(
            "Please specify which book to write in (e.g., on book-3).".to_string(),
        );
    }

    let book_in_cabin = matches!(state.player.room, Some(Room::CabinMain))
        && state
            .cabin_state()
            .map(|c| c.book_ids.iter().any(|b| b == &book_id))
            .unwrap_or(false);
    if !state.player_has_book(&book_id) && !book_in_cabin {
        return InteractionResult::Failure(
            "You need to hold the book (or be next to it in the cabin) to write in it.".to_string(),
        );
    }

    let Some(book) = state.book_entry_mut(&book_id) else {
        return InteractionResult::Failure("That book ID doesn't exist.".to_string());
    };
    if !book.writable {
        return InteractionResult::Failure("This book cannot be written in.".to_string());
    }

    book.set_page(page_num - 1, body.trim());
    InteractionResult::ActionSuccess {
        message: format!(
            "You write on page {} of {} ({})",
            page_num, book.title, book.id
        ),
        time_cost: 1,
        energy_cost: 1.0,
    }
}
