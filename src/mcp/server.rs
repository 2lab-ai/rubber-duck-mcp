use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, Write};

use super::protocol::*;
use super::tools::*;
use crate::actions::*;
use crate::descriptions::*;
use crate::entity::*;
use crate::persistence::*;
use crate::world::*;

pub struct McpServer {
    world: World,
    initialized: bool,
    log_path: std::path::PathBuf,
}

impl McpServer {
    pub fn new(state_path: std::path::PathBuf, log_path: std::path::PathBuf) -> Self {
        Self {
            world: World::new(state_path),
            initialized: false,
            log_path,
        }
    }

    /// Run the MCP server, reading from stdin and writing to stdout
    pub fn run(&mut self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        tracing::info!("MCP Server starting...");

        for line in stdin.lock().lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            tracing::debug!("Received: {}", line);

            let response = self.handle_message(&line);

            let response_json = serde_json::to_string(&response)?;
            tracing::debug!("Sending: {}", response_json);

            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;

            // Save state after each interaction
            if let Err(e) = self.world.save() {
                tracing::warn!("Failed to save state: {}", e);
            }
        }

        Ok(())
    }

    fn handle_message(&mut self, message: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(message) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to parse request: {}", e);
                return JsonRpcResponse::error(None, JsonRpcError::parse_error());
            }
        };

        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => self.handle_initialize(id, request.params),
            "initialized" => {
                // Notification, no response needed
                JsonRpcResponse::success(id, json!({}))
            }
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, request.params),
            method => {
                tracing::warn!("Unknown method: {}", method);
                JsonRpcResponse::error(id, JsonRpcError::method_not_found(method))
            }
        }
    }

    fn handle_initialize(&mut self, id: Option<Value>, _params: Option<Value>) -> JsonRpcResponse {
        self.initialized = true;

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
            },
            server_info: ServerInfo {
                name: "rubber-duck-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    fn handle_tools_list(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = get_tool_definitions();
        let result = ToolsListResult { tools };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    fn handle_tools_call(&mut self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let call_params: CallToolParams = match params.and_then(|p| serde_json::from_value(p).ok())
        {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing tool parameters"),
                );
            }
        };

        let result = self.execute_tool(&call_params.name, &call_params.arguments);
        if let Some(text) = extract_text(&result) {
            self.append_web_log(&format!("[{}] {}", call_params.name, text));
        }

        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    fn execute_tool(&mut self, name: &str, args: &Option<Value>) -> CallToolResult {
        let result = match name {
            "look" => self.cmd_look(args),
            "move" => self.cmd_move(args),
            "enter" => self.cmd_enter(args),
            "exit" => self.cmd_exit(args),
            "examine" => self.cmd_examine(args),
            "take" => self.cmd_take(args),
            "drop" => self.cmd_drop(args),
            "use" => self.cmd_use(args),
            "fish" => self.cmd_fish(args),
            "create" => self.cmd_create(args),
            "write" => self.cmd_write(args),
            "open" => self.cmd_open(args),
            "close" => self.cmd_close(args),
            "inventory" => self.cmd_inventory(args),
            "status" => self.cmd_status(args),
            "meditate" => self.cmd_meditate(args),
            "drink" => self.cmd_drink(args),
            "sleep" => self.cmd_sleep(args),
            "wait" => self.cmd_wait(args),
            "kick" => self.cmd_kick(args),
            "talk" => self.cmd_talk(args),
            "name" => self.cmd_name(args),
            "simulate" => self.cmd_simulate(args),
            "time" => self.cmd_time(args),
            "skills" => self.cmd_skills(args),
            _ => CallToolResult::error(format!("Unknown tool: {}", name)),
        };

        // Append any pending messages (like fire warnings)
        self.append_pending_messages(result)
    }

    fn append_pending_messages(&mut self, mut result: CallToolResult) -> CallToolResult {
        if !self.world.state.pending_messages.is_empty() {
            let messages = self
                .world
                .state
                .pending_messages
                .drain(..)
                .collect::<Vec<_>>();
            if let Some(ToolContent::Text { text }) = result.content.first_mut() {
                let notifications = messages.join("\n");
                *text = format!("{}\n\n**[{}]**", text, notifications);
            }
        }
        result
    }

    fn is_near_water(&self) -> bool {
        let pr = self.world.state.player.position.row;
        let pc = self.world.state.player.position.col;
        for dr in -1..=1 {
            for dc in -1..=1 {
                let pos = Position::new(pr + dr, pc + dc);
                if !pos.is_valid() {
                    continue;
                }
                if let Some((r, c)) = pos.as_usize() {
                    if let Some(tile) = self.world.map.get_tile(r, c) {
                        if matches!(tile.biome, Biome::Lake | Biome::Oasis) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // Command implementations

    fn cmd_look(&self, args: &Option<Value>) -> CallToolResult {
        let direction = get_string_arg(args, "direction");

        let text = if let Some(dir_str) = direction {
            if let Some(dir) = Direction::from_str(&dir_str) {
                DescriptionGenerator::look_direction(
                    dir,
                    &self.world.state.player,
                    &self.world.map,
                    &self.world.state.time,
                    &self.world.state.weather,
                    &self.world.state.wildlife,
                    &self.world.state.objects,
                )
            } else {
                format!("'{}' is not a valid direction.", dir_str)
            }
        } else {
            DescriptionGenerator::describe_location(
                &self.world.state.player,
                &self.world.map,
                &self.world.state.time,
                &self.world.state.weather,
                &self.world.state.wildlife,
                &self.world.state.objects,
            )
        };

        CallToolResult::text(text)
    }

    fn cmd_move(&mut self, args: &Option<Value>) -> CallToolResult {
        let dir_str = match get_string_arg(args, "direction") {
            Some(d) => d,
            None => {
                return CallToolResult::error("Please specify a direction to move.".to_string())
            }
        };

        let dir = match Direction::from_str(&dir_str) {
            Some(d) => d,
            None => {
                return CallToolResult::error(format!("'{}' is not a valid direction.", dir_str))
            }
        };

        let cabin_open = self
            .world
            .state
            .cabin_state()
            .map(|c| c.door_open)
            .unwrap_or(false);

        let result = try_move(
            &mut self.world.state.player,
            dir,
            &self.world.map,
            &self.world.state.objects,
            cabin_open,
        );

        // Tick the world after movement
        self.world.tick();

        // Possibly trigger one-time cabin tutorial hint when entering the cabin
        self.world.state.maybe_trigger_tutorial_hint();

        let text = match result {
            MoveResult::Success(msg) => {
                let location_desc = DescriptionGenerator::describe_location(
                    &self.world.state.player,
                    &self.world.map,
                    &self.world.state.time,
                    &self.world.state.weather,
                    &self.world.state.wildlife,
                    &self.world.state.objects,
                );
                format!("{}\n\n{}", msg, location_desc)
            }
            MoveResult::Blocked(msg) => msg,
            MoveResult::InvalidDirection(msg) => msg,
            MoveResult::RoomTransition(msg) => {
                let location_desc = DescriptionGenerator::describe_location(
                    &self.world.state.player,
                    &self.world.map,
                    &self.world.state.time,
                    &self.world.state.weather,
                    &self.world.state.wildlife,
                    &self.world.state.objects,
                );
                format!("{}\n\n{}", msg, location_desc)
            }
        };

        CallToolResult::text(text)
    }

    fn cmd_enter(&mut self, args: &Option<Value>) -> CallToolResult {
        let location = match get_string_arg(args, "location") {
            Some(l) => l,
            None => {
                return CallToolResult::error("Please specify a location to enter.".to_string())
            }
        };

        let cabin_open = self
            .world
            .state
            .cabin_state()
            .map(|c| c.door_open)
            .unwrap_or(false);
        let result = try_enter(
            &mut self.world.state.player,
            &location,
            &self.world.map,
            &self.world.state.objects,
            cabin_open,
        );

        // If we just entered the cabin, surface the tutorial hint once
        self.world.state.maybe_trigger_tutorial_hint();

        let text = match result {
            MoveResult::Success(msg) | MoveResult::RoomTransition(msg) => {
                let location_desc = DescriptionGenerator::describe_location(
                    &self.world.state.player,
                    &self.world.map,
                    &self.world.state.time,
                    &self.world.state.weather,
                    &self.world.state.wildlife,
                    &self.world.state.objects,
                );
                format!("{}\n\n{}", msg, location_desc)
            }
            MoveResult::Blocked(msg) | MoveResult::InvalidDirection(msg) => msg,
        };

        CallToolResult::text(text)
    }

    fn cmd_exit(&mut self, _args: &Option<Value>) -> CallToolResult {
        let result = try_exit(&mut self.world.state.player);

        let text = match result {
            MoveResult::RoomTransition(msg) => {
                let location_desc = DescriptionGenerator::describe_location(
                    &self.world.state.player,
                    &self.world.map,
                    &self.world.state.time,
                    &self.world.state.weather,
                    &self.world.state.wildlife,
                    &self.world.state.objects,
                );
                format!("{}\n\n{}", msg, location_desc)
            }
            MoveResult::InvalidDirection(msg) => msg,
            _ => "Unexpected result".to_string(),
        };

        CallToolResult::text(text)
    }

    fn cmd_examine(&self, args: &Option<Value>) -> CallToolResult {
        let target = match get_string_arg(args, "target") {
            Some(t) => t,
            None => return CallToolResult::error("Please specify what to examine.".to_string()),
        };

        let text = examine(&target, &self.world.state);

        CallToolResult::text(text)
    }

    fn cmd_take(&mut self, args: &Option<Value>) -> CallToolResult {
        let item = match get_string_arg(args, "item") {
            Some(i) => i,
            None => return CallToolResult::error("Please specify an item to take.".to_string()),
        };

        let result = try_take(&item, &mut self.world.state, &mut self.world.map);

        let text = match result {
            InteractionResult::Success(msg) => msg,
            InteractionResult::Failure(msg) => msg,
            InteractionResult::ItemObtained(_, msg) => msg,
            InteractionResult::ItemLost(_, msg) => msg,
            _ => "Action not supported here".to_string(),
        };

        CallToolResult::text(text)
    }

    fn cmd_drop(&mut self, args: &Option<Value>) -> CallToolResult {
        let item = match get_string_arg(args, "item") {
            Some(i) => i,
            None => return CallToolResult::error("Please specify an item to drop.".to_string()),
        };

        let result = try_drop(&item, &mut self.world.state, &mut self.world.map);

        let text = match result {
            InteractionResult::Success(msg) => msg,
            InteractionResult::Failure(msg) => msg,
            InteractionResult::ItemObtained(_, msg) => msg,
            InteractionResult::ItemLost(_, msg) => msg,
            _ => "Action not supported here".to_string(),
        };

        CallToolResult::text(text)
    }

    fn cmd_use(&mut self, args: &Option<Value>) -> CallToolResult {
        let item = match get_string_arg(args, "item") {
            Some(i) => i,
            None => return CallToolResult::error("Please specify an item to use.".to_string()),
        };

        let target = get_string_arg(args, "target");

        // Universal Use Handler from interaction.rs
        let result = try_use(
            &item,
            target.as_deref(),
            &mut self.world.state,
            &mut self.world.map,
        );

        match result {
            InteractionResult::Success(msg) => CallToolResult::text(msg),
            InteractionResult::Failure(msg) => CallToolResult::text(msg),
            InteractionResult::ItemObtained(_, msg) => CallToolResult::text(msg),
            InteractionResult::ItemLost(_, msg) => CallToolResult::text(msg),
            InteractionResult::ActionSuccess {
                message,
                time_cost,
                energy_cost,
            } => {
                // Pass time and drain energy
                for _ in 0..time_cost {
                    self.world.tick();
                }
                self.world.state.player.modify_energy(-energy_cost);

                let time_str = if time_cost > 0 {
                    format!(" (took {} mins)", time_cost * 10)
                } else {
                    "".to_string()
                };
                CallToolResult::text(format!("{}{}", message, time_str))
            }
        }
    }

    fn cmd_create(&mut self, args: &Option<Value>) -> CallToolResult {
        let item = match get_string_arg(args, "item") {
            Some(i) => i,
            None => return CallToolResult::error("Please specify an item to create.".to_string()),
        };

        let result = try_create(&item, &mut self.world.state);

        match result {
            InteractionResult::Success(msg) => CallToolResult::text(msg),
            InteractionResult::Failure(msg) => CallToolResult::text(msg),
            _ => CallToolResult::error("Unexpected result".to_string()),
        }
    }

    fn cmd_write(&mut self, args: &Option<Value>) -> CallToolResult {
        let text = match get_string_arg(args, "text") {
            Some(t) => t,
            None => {
                return CallToolResult::error(
                    "Please provide text to write (제목: or 페이지N:).".to_string(),
                )
            }
        };
        let target = match get_string_arg(args, "target") {
            Some(t) => t,
            None => {
                return CallToolResult::error(
                    "Please specify a target book (e.g., '빈 책' or 'book-1').".to_string(),
                )
            }
        };

        let result = write_on_book(&text, &target, &mut self.world.state);

        match result {
            InteractionResult::Success(msg) => CallToolResult::text(msg),
            InteractionResult::Failure(msg) => CallToolResult::text(msg),
            InteractionResult::ActionSuccess {
                message,
                time_cost,
                energy_cost,
            } => {
                for _ in 0..time_cost {
                    self.world.tick();
                }
                self.world.state.player.modify_energy(-energy_cost);
                let time_str = if time_cost > 0 {
                    format!(" (took {} mins)", time_cost * 10)
                } else {
                    "".to_string()
                };
                CallToolResult::text(format!("{}{}", message, time_str))
            }
            _ => CallToolResult::error("Unexpected result".to_string()),
        }
    }

    fn cmd_open(&mut self, args: &Option<Value>) -> CallToolResult {
        let target = match get_string_arg(args, "target") {
            Some(t) => t,
            None => return CallToolResult::error("Please specify what to open.".to_string()),
        };

        let result = try_open(&target, &mut self.world.state);

        let text = match result {
            InteractionResult::Success(msg) => msg,
            InteractionResult::Failure(msg) => msg,
            _ => "Unexpected result".to_string(),
        };

        CallToolResult::text(text)
    }

    fn cmd_close(&mut self, args: &Option<Value>) -> CallToolResult {
        let target = match get_string_arg(args, "target") {
            Some(t) => t,
            None => return CallToolResult::error("Please specify what to close.".to_string()),
        };

        let result = try_close(&target, &mut self.world.state);

        let text = match result {
            InteractionResult::Success(msg) => msg,
            InteractionResult::Failure(msg) => msg,
            _ => "Unexpected result".to_string(),
        };

        CallToolResult::text(text)
    }

    fn cmd_inventory(&self, _args: &Option<Value>) -> CallToolResult {
        let items = self.world.state.player.inventory.list();

        if items.is_empty() {
            return CallToolResult::text("You are not carrying anything.".to_string());
        }

        let mut text = String::from("**Inventory:**\n");
        for (item, qty) in items {
            if qty == 1 {
                text.push_str(&format!("- {}\n", item.name()));
            } else {
                text.push_str(&format!("- {} (x{})\n", item.name(), qty));
            }
        }

        // Show active project if any
        if let Some(bp) = &self.world.state.player.active_project {
            text.push_str(&format!(
                "\n**Active Project:**\n- {}\n",
                bp.status_description()
            ));
        }

        if !self.world.state.player.book_ids.is_empty() {
            text.push_str("\n**Books:**\n");
            for id in &self.world.state.player.book_ids {
                if let Some(book) = self.world.state.books.get(id) {
                    text.push_str(&format!("- {} ({})\n", book.title, book.id));
                } else {
                    text.push_str(&format!("- {}\n", id));
                }
            }
        }

        let weight = self.world.state.player.inventory.current_weight();
        let max_weight = self.world.state.player.inventory.max_weight;
        text.push_str(&format!("\nCarrying: {:.1}/{:.1} kg", weight, max_weight));

        CallToolResult::text(text)
    }

    fn cmd_status(&self, _args: &Option<Value>) -> CallToolResult {
        let player = &self.world.state.player;

        let text = format!(
            "**Your Status:**\n\n\
            Health: {:.0}/100\n\
            Warmth: {:.0}/100 ({})\n\
            Energy: {:.0}/100 ({})\n\
            Mood: {:.0}/100 ({})\n\
            Fullness: {:.0}/100 ({})\n\
            Hydration: {:.0}/100 ({})\n\n\
            {}",
            player.health,
            player.warmth,
            player.comfort_description(),
            player.energy,
            player.energy_description(),
            player.mood,
            player.mood_description(),
            player.fullness,
            player.fullness_description(),
            player.hydration,
            player.hydration_description(),
            player.status_summary()
        );

        CallToolResult::text(text)
    }

    fn cmd_meditate(&mut self, _args: &Option<Value>) -> CallToolResult {
        let position = self.world.state.player.position;
        let room = self.world.state.player.room.clone();

        let near_water = self.is_near_water();
        let cozy_fire = matches!(room, Some(Room::CabinMain))
            && self
                .world
                .state
                .cabin_state()
                .map(|c| !matches!(c.fireplace.state, FireState::Cold))
                .unwrap_or(false);

        let (row, col) = position.as_usize().unwrap_or((5, 5));
        let biome = self
            .world
            .map
            .get_biome_at(row, col)
            .unwrap_or(Biome::MixedForest);

        let setting = match room {
            Some(Room::CabinMain) if cozy_fire => {
                "You settle near the fireplace, letting its warmth seep into your hands."
            }
            Some(Room::CabinMain) => "You find a quiet corner of the cabin and close your eyes.",
            Some(Room::CabinTerrace) => {
                "You rest on the terrace rail, eyes drifting over the lake's surface."
            }
            Some(Room::WoodShed) => {
                "You lean against the shed wall, breathing in the scent of cut wood."
            }
            None if near_water => "You sit by the water's edge, watching ripples form and fade.",
            None => "You find a soft patch of ground and sit cross-legged, grounding yourself.",
        };

        // Let a little time pass while meditating
        self.world.tick();

        let mut mood_gain = 12.0;
        if near_water {
            mood_gain += 3.0;
        }
        if cozy_fire {
            mood_gain += 2.0;
        }

        let energy_gain = 5.0;
        let warmth_gain = if cozy_fire { 6.0 } else { 0.0 };

        let player = &mut self.world.state.player;
        player.modify_mood(mood_gain);
        player.modify_energy(energy_gain);
        if warmth_gain > 0.0 {
            player.modify_warmth(warmth_gain);
        }

        let sky_desc = describe_sky(
            &self.world.state.time,
            &self.world.state.weather,
            position.row,
            position.col,
            biome,
        );
        let time_desc = self.world.state.time.time_description();

        let texture = if cozy_fire {
            "The steady crackle of the fire keeps you anchored in the moment."
        } else if near_water {
            "Waves lap softly nearby, keeping time with your breath."
        } else {
            "The quiet around you makes it easy to notice each inhale and exhale."
        };

        let text = format!(
            "{}

{}
{}

You breathe slowly, letting thoughts drift. After a few minutes, a gentle clarity settles in.

You feel calmer and a bit more refreshed. It is now {}.",
            setting,
            sky_desc.trim(),
            texture,
            time_desc
        );

        CallToolResult::text(text)
    }

    fn cmd_drink(&mut self, _args: &Option<Value>) -> CallToolResult {
        let near_water = self.is_near_water();
        if !near_water {
            return CallToolResult::error(
                "You need to be right by the lake to drink the water.".to_string(),
            );
        }

        self.world.state.player.modify_hydration(30.0);
        self.world.state.player.modify_fullness(3.0);
        self.world.state.player.modify_mood(2.0);

        // A quick sip still passes a little time
        self.world.tick();

        CallToolResult::text(
            "You kneel and cup cold lake water in your hands, drinking deeply. It tastes clean and refreshing.".to_string()
        )
    }

    fn cmd_fish(&mut self, args: &Option<Value>) -> CallToolResult {
        let gear = get_string_arg(args, "gear");
        let result = try_fish(&mut self.world.state, &self.world.map, gear.as_deref());

        match result {
            InteractionResult::Success(msg) | InteractionResult::Failure(msg) => {
                CallToolResult::text(msg)
            }
            InteractionResult::ItemObtained(_, msg) | InteractionResult::ItemLost(_, msg) => {
                CallToolResult::text(msg)
            }
            InteractionResult::ActionSuccess {
                message,
                time_cost,
                energy_cost,
            } => {
                for _ in 0..time_cost {
                    self.world.tick();
                }
                self.world.state.player.modify_energy(-energy_cost);
                let time_str = if time_cost > 0 {
                    format!(" (took {} mins)", time_cost * 10)
                } else {
                    "".to_string()
                };
                CallToolResult::text(format!("{}{}", message, time_str))
            }
        }
    }

    fn cmd_sleep(&mut self, _args: &Option<Value>) -> CallToolResult {
        let well_fed = {
            let p = &self.world.state.player;
            p.fullness >= 60.0 && p.hydration >= 50.0
        };

        // Advance time while sleeping (about an hour)
        for _ in 0..6 {
            self.world.tick();
        }

        // Restore stats
        let player = &mut self.world.state.player;
        player.modify_energy(25.0);
        player.modify_mood(6.0);
        player.modify_fullness(-5.0);
        player.modify_hydration(-5.0);
        if well_fed {
            player.modify_health(15.0);
        } else {
            player.modify_health(5.0);
        }

        let text = if well_fed {
            "You curl up and drift into a deep, satisfying sleep. With a full belly and quenched thirst, your body mends itself."
        } else {
            "You doze for a while. It's not the most comfortable rest, but it helps a bit."
        };

        CallToolResult::text(format!("{}\n\nYou wake feeling more rested.", text))
    }

    fn cmd_wait(&mut self, args: &Option<Value>) -> CallToolResult {
        let duration = get_string_arg(args, "duration").unwrap_or_else(|| "short".to_string());

        let ticks = match duration.as_str() {
            "short" => 1,
            "medium" => 3,
            "long" => 6,
            _ => 1,
        };

        for _ in 0..ticks {
            self.world.tick();
        }

        let time_desc = self.world.state.time.time_description();

        // Get a random wildlife description if any nearby
        let mut wildlife_note = String::new();
        let nearby: Vec<_> = self
            .world
            .state
            .wildlife
            .iter()
            .filter(|w| w.position.distance_to(&self.world.state.player.position) < 4.0)
            .collect();

        if !nearby.is_empty() {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            if let Some(w) = nearby.choose(&mut rng) {
                wildlife_note = format!("\n\n{}", w.describe());
            }
        }

        let text = format!(
            "Time passes...\n\nIt is now {}.{}",
            time_desc, wildlife_note
        );

        CallToolResult::text(text)
    }

    fn cmd_kick(&mut self, _args: &Option<Value>) -> CallToolResult {
        let result = kick_tree(&mut self.world.state);

        let text = match result {
            CraftResult::Success(msg) => msg,
            CraftResult::Failure(msg) => msg,
            CraftResult::PartialSuccess(msg) => msg,
        };

        CallToolResult::text(text)
    }

    fn cmd_talk(&mut self, args: &Option<Value>) -> CallToolResult {
        let message = get_string_arg(args, "message");
        let duck_name = self.world.state.display_name(&Item::RubberDuck);
        let result = talk_to_animal_companion(message.as_deref(), &self.world.state)
            .unwrap_or_else(|| {
                talk_to_rubber_duck(message.as_deref(), &self.world.state, &duck_name)
            });

        let text = match result {
            InteractionResult::Success(msg) => msg,
            InteractionResult::Failure(msg) => msg,
            InteractionResult::ItemObtained(_, msg) => msg,
            InteractionResult::ItemLost(_, msg) => msg,
            _ => "Action not supported.".to_string(),
        };

        CallToolResult::text(text)
    }

    fn cmd_name(&mut self, args: &Option<Value>) -> CallToolResult {
        let item_str = match get_string_arg(args, "item") {
            Some(i) => i,
            None => return CallToolResult::error("Please specify which item to name.".to_string()),
        };
        let new_name = match get_string_arg(args, "name") {
            Some(n) => n,
            None => return CallToolResult::error("Please provide a name.".to_string()),
        };

        let item = match Item::from_str(&item_str) {
            Some(i) => i,
            None => return CallToolResult::error(format!("Unknown item '{}'.", item_str)),
        };

        if !self.world.state.player_can_access_item(&item) {
            return CallToolResult::error(
                "You need to have or be next to that item to name it.".to_string(),
            );
        }

        self.world.state.set_custom_name(item, &new_name);
        let display = self.world.state.display_name(&item);
        CallToolResult::text(format!("You name the {} '{}'.", item.name(), display))
    }

    fn cmd_simulate(&mut self, args: &Option<Value>) -> CallToolResult {
        let ticks = get_int_arg(args, "ticks", 1).clamp(1, 10) as usize;

        for _ in 0..ticks {
            self.world.tick();
        }

        let time_desc = self.world.state.time.time_description();
        let text = format!(
            "The world advances {} tick(s).\n\nIt is now {}.",
            ticks, time_desc
        );

        CallToolResult::text(text)
    }

    fn cmd_time(&self, _args: &Option<Value>) -> CallToolResult {
        let time = &self.world.state.time;
        let weather = &self.world.state.weather;
        let player_pos = &self.world.state.player.position;

        let current_weather = weather.get_for_position(player_pos.row, player_pos.col);

        let text = format!(
            "**Time:** {}\n\
            **Day:** {}\n\
            **Weather:** {}",
            time.time_description(),
            time.day,
            current_weather.name()
        );

        CallToolResult::text(text)
    }

    fn cmd_skills(&self, _args: &Option<Value>) -> CallToolResult {
        let skills = &self.world.state.player.skills;

        let text = format!(
            "**Skills:**\n\n\
            Woodcutting: {}/100\n\
            Fire Making: {}/100\n\
            Observation: {}/100\n\
            Foraging: {}/100\n\
            Stonemasonry: {}/100\n\
            Survival: {}/100\n\
            Tailoring: {}/100\n\
            Cooking: {}/100",
            skills.woodcutting,
            skills.fire_making,
            skills.observation,
            skills.foraging,
            skills.stonemasonry,
            skills.survival,
            skills.tailoring,
            skills.cooking
        );

        CallToolResult::text(text)
    }

    fn append_web_log(&self, line: &str) {
        use std::fs::OpenOptions;
        use std::io::Write;

        if let Some(parent) = self.log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(f, "[{}] {}", timestamp(), line);
        }
    }
}

fn extract_text(result: &CallToolResult) -> Option<String> {
    result.content.iter().find_map(|c| match c {
        ToolContent::Text { text } => Some(text.clone()),
    })
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}
