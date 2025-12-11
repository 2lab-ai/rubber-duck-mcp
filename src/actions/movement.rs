use crate::entity::{Player, Room};
use crate::world::{Direction, ObjectKind, ObjectRegistry, Position, TileType, WorldMap};

pub enum MoveResult {
    Success(String),
    Blocked(String),
    InvalidDirection(String),
    RoomTransition(String),
}

/// Move the player in a direction
pub fn try_move(
    player: &mut Player,
    dir: Direction,
    map: &WorldMap,
    objects: &ObjectRegistry,
    cabin_open: bool,
) -> MoveResult {
    // If in a room, movement works differently
    if let Some(room) = &player.room {
        return handle_room_movement(player, dir, room.clone(), cabin_open);
    }

    let new_pos = player.position.move_in_direction(dir);

    // Special-case: prevent deeper cave exploration past the entrance for now
    if objects
        .objects_at(&player.position)
        .iter()
        .any(|o| o.id == "east_cave_entrance")
        && matches!(dir, Direction::East)
    {
        return MoveResult::Blocked(
            "The cave beyond is pitch black. Without a reliable light source and proper gear, you don't dare go any deeper."
                .to_string(),
        );
    }

    // Check bounds
    if !new_pos.is_valid() {
        return MoveResult::Blocked(
            "You cannot go that way - the path ends at the edge of the world.".to_string(),
        );
    }

    let (row, col) = new_pos.as_usize().unwrap();

    // Check if walkable
    if !map.is_walkable(row, col) {
        let tile = map.get_tile(row, col);
        let reason = match tile.map(|t| &t.tile_type) {
            Some(TileType::Lake) => {
                "The lake's cool waters block your path. You would need a boat to continue."
            }
            _ => "Something blocks your way.",
        };
        return MoveResult::Blocked(reason.to_string());
    }

    // Check cabin entrance via objects
    if objects
        .objects_at(&new_pos)
        .iter()
        .any(|o| matches!(o.object.kind, ObjectKind::Cabin(_)))
    {
        if !cabin_open {
            return MoveResult::Blocked(
                "You stand before the cabin. The wooden door is closed. Perhaps you should try to open it.".to_string()
            );
        }
        player.position = new_pos;
        player.mark_visited();
        player.enter_room(Room::CabinMain);
        return MoveResult::RoomTransition(
            "You push open the door and step into the cabin.".to_string(),
        );
    }

    // Normal movement
    player.position = new_pos;
    player.mark_visited();
    player.face(dir);
    // Basic movement always works, but injured legs make it slower and more tiring.
    let movement_factor = player.body.movement_factor();
    let (verb, energy_cost) = if movement_factor >= 0.9 {
        ("walk", 1.0)
    } else if movement_factor >= 0.6 {
        ("walk carefully", 1.5)
    } else {
        ("limp", 2.0)
    };
    player.modify_energy(-energy_cost);

    MoveResult::Success(format!("You {} {}.", verb, dir_name(dir)))
}

fn handle_room_movement(
    player: &mut Player,
    dir: Direction,
    current_room: Room,
    _cabin_open: bool,
) -> MoveResult {
    match (&current_room, dir) {
        // From cabin main room
        (Room::CabinMain, Direction::South) => {
            player.exit_room();
            player.face(Direction::South);
            MoveResult::RoomTransition(
                "You step out through the cabin door into the cool air.".to_string(),
            )
        }
        (Room::CabinMain, Direction::North) => {
            player.room = Some(Room::CabinTerrace);
            MoveResult::RoomTransition(
                "You walk through to the back terrace overlooking the lake.".to_string(),
            )
        }
        (Room::CabinMain, Direction::West) => {
            player.room = Some(Room::WoodShed);
            MoveResult::RoomTransition(
                "You exit through the side door into the wood shed.".to_string(),
            )
        }

        // From terrace
        (Room::CabinTerrace, Direction::South) => {
            player.room = Some(Room::CabinMain);
            MoveResult::RoomTransition("You step back into the warmth of the cabin.".to_string())
        }
        (Room::CabinTerrace, Direction::West) => {
            player.room = Some(Room::WoodShed);
            MoveResult::RoomTransition("You walk around to the wood shed.".to_string())
        }

        // From wood shed
        (Room::WoodShed, Direction::East) => {
            player.room = Some(Room::CabinMain);
            MoveResult::RoomTransition("You return to the cabin's main room.".to_string())
        }
        (Room::WoodShed, Direction::North) => {
            player.room = Some(Room::CabinTerrace);
            MoveResult::RoomTransition("You walk around to the terrace.".to_string())
        }
        (Room::WoodShed, Direction::South) => {
            player.exit_room();
            player.face(Direction::South);
            MoveResult::RoomTransition("You exit the wood shed and return outside.".to_string())
        }

        _ => MoveResult::Blocked("You can't go that way from here.".to_string()),
    }
}

fn dir_name(dir: Direction) -> &'static str {
    match dir {
        Direction::North => "north",
        Direction::South => "south",
        Direction::East => "east",
        Direction::West => "west",
        Direction::Up => "up",
        Direction::Down => "down",
    }
}

/// Enter a location at current position
pub fn try_enter(
    player: &mut Player,
    target: &str,
    _map: &WorldMap,
    objects: &ObjectRegistry,
    cabin_open: bool,
) -> MoveResult {
    let normalized = target.to_lowercase();
    let cabin_pos = objects
        .find("cabin")
        .map(|p| p.position)
        .unwrap_or_else(|| Position::new(6, 5));

    // Check if trying to enter cabin (either on cabin tile or adjacent to it)
    if normalized.contains("cabin") || normalized.contains("door") || normalized.contains("house") {
        let distance = player.position.distance_to(&cabin_pos);

        // Must be on or adjacent to cabin
        if distance > 1.5 {
            return MoveResult::InvalidDirection(
                "You're too far from the cabin to enter it.".to_string(),
            );
        }

        if !cabin_open {
            return MoveResult::Blocked(
                "The cabin door is closed. You need to open it first.".to_string(),
            );
        }

        player.position = cabin_pos; // Move to cabin position
        player.enter_room(Room::CabinMain);
        return MoveResult::RoomTransition("You step into the cozy cabin.".to_string());
    }

    // Check for entering wood shed from outside
    if player.room.is_none() {
        if let Some(shed_pos) = objects.find("wood_shed").map(|p| p.position) {
            if player.position.distance_to(&shed_pos) < 2.0
                && (normalized.contains("shed") || normalized.contains("wood"))
            {
                player.enter_room(Room::WoodShed);
                return MoveResult::RoomTransition("You enter the small wood shed.".to_string());
            }
        }
    }

    // Check for entering the east-side cave entrance from outside
    if player.room.is_none() && normalized.contains("cave") {
        if let Some(cave) = objects.find("east_cave_entrance") {
            let cave_pos = cave.position;
            let distance = player.position.distance_to(&cave_pos);
            if distance > 1.5 {
                return MoveResult::InvalidDirection(
                    "You're too far from the cave entrance to step inside.".to_string(),
                );
            }

            player.position = cave_pos;
            player.mark_visited();
            return MoveResult::Success(
                "You step into the mouth of the cave. Just beyond the entrance, darkness swallows the passage; without proper light and gear, you decide not to go any deeper yet."
                    .to_string(),
            );
        }
    }

    MoveResult::InvalidDirection(format!("You don't see a '{}' to enter here.", target))
}

/// Exit current interior location
pub fn try_exit(player: &mut Player) -> MoveResult {
    if player.room.is_some() {
        player.exit_room();
        player.face(Direction::South);
        MoveResult::RoomTransition("You step outside.".to_string())
    } else {
        MoveResult::InvalidDirection("You are already outside.".to_string())
    }
}
