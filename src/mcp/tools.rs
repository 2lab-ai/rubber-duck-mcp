use serde_json::{json, Value};
use super::protocol::ToolDefinition;

/// Get all available tool definitions
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "look".to_string(),
            description: "Observe your surroundings. Without a direction, describes your current location in detail. With a direction (north/south/east/west), describes what you see in that direction.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "direction": {
                        "type": "string",
                        "description": "Optional direction to look: north, south, east, west",
                        "enum": ["north", "south", "east", "west", "n", "s", "e", "w"]
                    }
                }
            }),
        },
        ToolDefinition {
            name: "move".to_string(),
            description: "Move in a direction. Use to navigate the world and explore different areas.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "direction": {
                        "type": "string",
                        "description": "Direction to move: north, south, east, west",
                        "enum": ["north", "south", "east", "west", "n", "s", "e", "w"]
                    }
                },
                "required": ["direction"]
            }),
        },
        ToolDefinition {
            name: "enter".to_string(),
            description: "Enter a building or location, such as the cabin or wood shed.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The location to enter (e.g., 'cabin', 'shed')"
                    }
                },
                "required": ["location"]
            }),
        },
        ToolDefinition {
            name: "exit".to_string(),
            description: "Exit the current indoor location and return outside.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "examine".to_string(),
            description: "Closely examine an object, creature, or feature for more detail.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "What to examine (e.g., 'fireplace', 'axe', 'strange compass')"
                    }
                },
                "required": ["target"]
            }),
        },
        ToolDefinition {
            name: "take".to_string(),
            description: "Pick up an item and add it to your inventory.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "item": {
                        "type": "string",
                        "description": "The item to pick up"
                    }
                },
                "required": ["item"]
            }),
        },
        ToolDefinition {
            name: "drop".to_string(),
            description: "Drop an item from your inventory.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "item": {
                        "type": "string",
                        "description": "The item to drop"
                    }
                },
                "required": ["item"]
            }),
        },
        ToolDefinition {
            name: "use".to_string(),
            description: "Use an item, optionally on a target. Examples: use axe on log, use matchbox on kindling, use blanket.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "item": {
                        "type": "string",
                        "description": "The item to use"
                    },
                    "target": {
                        "type": "string",
                        "description": "Optional target to use the item on"
                    }
                },
                "required": ["item"]
            }),
        },
        ToolDefinition {
            name: "open".to_string(),
            description: "Open a door or container.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "What to open (e.g., 'door', 'cabin door')"
                    }
                },
                "required": ["target"]
            }),
        },
        ToolDefinition {
            name: "close".to_string(),
            description: "Close a door or container.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "What to close"
                    }
                },
                "required": ["target"]
            }),
        },
        ToolDefinition {
            name: "inventory".to_string(),
            description: "List all items you are currently carrying.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "status".to_string(),
            description: "Check your current physical and mental condition, including warmth, energy, health, and mood.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "wait".to_string(),
            description: "Pass some time, allowing the world to change around you. Good for observing wildlife or waiting for conditions to change.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "duration": {
                        "type": "string",
                        "description": "How long to wait: short (10 min), medium (30 min), or long (1 hour)",
                        "enum": ["short", "medium", "long"],
                        "default": "short"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "simulate".to_string(),
            description: "Advance the world simulation by a number of ticks. Each tick is about 10 minutes of game time.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "ticks": {
                        "type": "integer",
                        "description": "Number of simulation ticks to advance (1-10)",
                        "minimum": 1,
                        "maximum": 10,
                        "default": 1
                    }
                }
            }),
        },
        ToolDefinition {
            name: "time".to_string(),
            description: "Check the current time of day and weather conditions.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "skills".to_string(),
            description: "View your current skill levels.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

/// Parse tool arguments helper
pub fn get_string_arg(args: &Option<Value>, key: &str) -> Option<String> {
    args.as_ref()
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn get_int_arg(args: &Option<Value>, key: &str, default: i64) -> i64 {
    args.as_ref()
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_i64())
        .unwrap_or(default)
}
