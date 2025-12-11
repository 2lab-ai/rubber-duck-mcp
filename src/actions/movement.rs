use crate::world::{Direction, Position, WorldMap, TileType, ObjectRegistry, ObjectKind};
use crate::entity::{Player, Room};

pub enum MoveResult {
    Success(String),
    Blocked(String),
    InvalidDirection(String),
    RoomTransition(String),
}

/// Move the player in a direction
pub fn try_move(player: &mut Player, dir: Direction, map: &WorldMap, objects: &ObjectRegistry, cabin_open: bool) -> MoveResult {
    // If in a room, movement works differently
    if let Some(room) = &player.room {
        return handle_room_movement(player, dir, room.clone(), cabin_open);
    }

    let new_pos = player.position.move_in_direction(dir);

    // Check bounds
    if !new_pos.is_valid() {
        return MoveResult::Blocked(
            "You cannot go that way - the path ends at the edge of the world.".to_string()
        );
    }

    let (row, col) = new_pos.as_usize().unwrap();

    // Check if walkable
    if !map.is_walkable(row, col) {
        let tile = map.get_tile(row, col);
        let reason = match tile.map(|t| &t.tile_type) {
            Some(TileType::Lake) => "The lake's cool waters block your path. You would need a boat to continue.",
            _ => "Something blocks your way.",
        };
        return MoveResult::Blocked(reason.to_string());
    }

    // Check cabin entrance via objects
    let target_pos = Position::new(row as i32, col as i32);
    if objects
        .objects_at(&target_pos)
        .iter()
        .any(|o| matches!(o.object.kind, ObjectKind::Cabin(_)))
    {
        if !cabin_open {
            return MoveResult::Blocked(
                "You stand before the cabin. The wooden door is closed. Perhaps you should try to open it.".to_string()
            );
        }
        player.position = new_pos;
        player.enter_room(Room::CabinMain);
        return MoveResult::RoomTransition(
            "You push open the door and step into the cabin.".to_string()
        );
    }

    // Normal movement
    player.position = new_pos;
    player.face(dir);
    player.modify_energy(-1.0); // Movement costs a little energy

    MoveResult::Success(format!("You walk {}.", dir_name(dir)))
}

fn handle_room_movement(player: &mut Player, dir: Direction, current_room: Room, _cabin_open: bool) -> MoveResult {
    match (&current_room, dir) {
        // From cabin main room
        (Room::CabinMain, Direction::South) => {
            player.exit_room();
            player.face(Direction::South);
            MoveResult::RoomTransition(
                "You step out through the cabin door into the cool air.".to_string()
            )
        }
        (Room::CabinMain, Direction::North) => {
            player.room = Some(Room::CabinTerrace);
            MoveResult::RoomTransition(
                "You walk through to the back terrace overlooking the lake.".to_string()
            )
        }
        (Room::CabinMain, Direction::West) => {
            player.room = Some(Room::WoodShed);
            MoveResult::RoomTransition(
                "You exit through the side door into the wood shed.".to_string()
            )
        }

        // From terrace
        (Room::CabinTerrace, Direction::South) => {
            player.room = Some(Room::CabinMain);
            MoveResult::RoomTransition(
                "You step back into the warmth of the cabin.".to_string()
            )
        }
        (Room::CabinTerrace, Direction::West) => {
            player.room = Some(Room::WoodShed);
            MoveResult::RoomTransition(
                "You walk around to the wood shed.".to_string()
            )
        }

        // From wood shed
        (Room::WoodShed, Direction::East) => {
            player.room = Some(Room::CabinMain);
            MoveResult::RoomTransition(
                "You return to the cabin's main room.".to_string()
            )
        }
        (Room::WoodShed, Direction::North) => {
            player.room = Some(Room::CabinTerrace);
            MoveResult::RoomTransition(
                "You walk around to the terrace.".to_string()
            )
        }
        (Room::WoodShed, Direction::South) => {
            player.exit_room();
            player.face(Direction::South);
            MoveResult::RoomTransition(
                "You exit the wood shed and return outside.".to_string()
            )
        }

        _ => MoveResult::Blocked(
            "You can't go that way from here.".to_string()
        ),
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
pub fn try_enter(player: &mut Player, target: &str, _map: &WorldMap, objects: &ObjectRegistry, cabin_open: bool) -> MoveResult {
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
                "You're too far from the cabin to enter it.".to_string()
            );
        }

        if !cabin_open {
            return MoveResult::Blocked(
                "The cabin door is closed. You need to open it first.".to_string()
            );
        }

        player.position = cabin_pos; // Move to cabin position
        player.enter_room(Room::CabinMain);
        return MoveResult::RoomTransition(
            "You step into the cozy cabin.".to_string()
        );
    }

    // Check for entering wood shed from outside
    if player.room.is_none() {
        if let Some(shed_pos) = objects.find("wood_shed").map(|p| p.position) {
            if player.position.distance_to(&shed_pos) < 2.0 && (normalized.contains("shed") || normalized.contains("wood")) {
                player.enter_room(Room::WoodShed);
                return MoveResult::RoomTransition(
                    "You enter the small wood shed.".to_string()
                );
            }
        }
    }

    MoveResult::InvalidDirection(format!("You don't see a '{}' to enter here.", target))
}

/// Exit current interior location
pub fn try_exit(player: &mut Player) -> MoveResult {
    if player.room.is_some() {
        player.exit_room();
        player.face(Direction::South);
        MoveResult::RoomTransition(
            "You step outside.".to_string()
        )
    } else {
        MoveResult::InvalidDirection(
            "You are already outside.".to_string()
        )
    }
}
