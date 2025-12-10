# Rubber Duck MCP

A text-based healing MUD (Multi-User Dungeon) nature simulation, delivered as an MCP (Model Context Protocol) server. Experience a serene 11x11 world where four seasons exist simultaneously in different directions.

## Features

- **Four Seasonal Biomes**:
  - West: Scorching desert with an oasis
  - North: Temperate spring/autumn forest with distant snow-capped mountains
  - East: Snowy winter wonderland with aurora borealis
  - South: Starting area with mixed woodland

- **Central Lake**: A tranquil body of water that reflects the seasons around it

- **Cozy Cabin**: A refuge from the elements with:
  - Fireplace that you can light for warmth
  - Curious artifacts to examine
  - Terrace overlooking the lake
  - Wood shed for chopping firewood

- **Dynamic Simulation**:
  - Day/night cycle with time-appropriate descriptions
  - Regional weather system
  - Wildlife that moves and behaves naturally
  - Temperature and comfort mechanics

- **Detailed Interactions**:
  - Chop wood with skill checks
  - Build and maintain fires
  - Observe wildlife
  - Explore rich, descriptive environments

## Installation

```bash
# Clone the repository
cd rubber-duck-mcp

# Build
cargo build --release

# The binary will be at target/release/rubber-duck-mcp
```

## Usage

### As an MCP Server

Add to your Claude Desktop configuration (`~/.claude/settings.json` or similar):

```json
{
  "mcpServers": {
    "rubber-duck": {
      "command": "/path/to/rubber-duck-mcp",
      "env": {
        "RUBBER_DUCK_STATE": "/path/to/your/save/world_state.json"
      }
    }
  }
}
```

### Available Tools

- `look` - Observe surroundings (optional: direction)
- `move` - Move in a direction (north/south/east/west)
- `enter` - Enter a building
- `exit` - Exit current indoor location
- `examine` - Look closely at something
- `take` - Pick up an item
- `drop` - Drop an item
- `use` - Use an item (optionally on a target)
- `open` / `close` - Interact with doors
- `inventory` - List carried items
- `status` - Check your condition
- `wait` - Pass time observing
- `simulate` - Advance world simulation
- `time` - Check current time and weather
- `skills` - View skill levels

## Gameplay Example

```
> look
As dawn breaks, you find yourself in the worn forest path.

You stand on a well-trodden dirt path through the forest.
The path continues north toward a cabin.

The eastern horizon glows with the soft pink and gold of dawn.
A rabbit nibbles on clover, nose twitching constantly.

**Exits:** North: the cabin (door closed) | South: mixed woodland

> open door
You grasp the worn iron handle and pull. The door swings open
with a soft creak, revealing the dim interior of the cabin.

> move north
You walk north.

You are in the main room of the cabin. Soft daylight filters
through the windows.

A stone fireplace dominates one wall. The fireplace is cold
and dark, filled only with old ash...
```

## State Persistence

The game state is automatically saved to `data/world_state.json` (or the path specified by `RUBBER_DUCK_STATE` environment variable). Your progress, inventory, and the state of the world persist between sessions.

## Architecture

```
rubber-duck-mcp/
├── src/
│   ├── main.rs           # Entry point
│   ├── mcp/              # MCP protocol implementation
│   ├── world/            # Map, time, weather systems
│   ├── entity/           # Player, wildlife, objects
│   ├── actions/          # Movement, interaction, crafting
│   ├── descriptions/     # Rich text generation
│   └── persistence/      # Save/load state
├── data/
│   └── world_state.json  # Game state
└── docs/
    └── GDD.md            # Game design document
```

## Development

```bash
# Build in debug mode
cargo build

# Run with logging
RUST_LOG=debug cargo run

# Build release
cargo build --release
```

## License

MIT
