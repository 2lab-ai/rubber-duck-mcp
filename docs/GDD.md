# Rubber Duck MCP - Game Design Document

## Overview

**Genre:** Text-based Healing MUD / Nature Simulation
**Platform:** MCP Server (stdio)
**Language:** Rust
**Concept:** A serene text adventure where an AI can explore a mystical 11x11 natural world with four seasons existing simultaneously in different directions. The goal is to provide a calming, contemplative experience through detailed environmental descriptions and mindful interactions.

---

## World Design

### Map Structure (11x11 Grid)

```
          NORTH (Spring/Autumn - Distant Snowy Mountains)
              ↑
    [Forest][Forest][Forest][Forest][Forest][Forest][Forest][Forest][Forest][Forest][Forest]
    [Forest][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Forest]
    [Forest][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Forest]
    [Forest][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Forest]
WEST[Desert][Oasis ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Snow  ][Snow ]EAST
    [Desert][Oasis ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Lake  ][Snow  ][Snow ]
    [Forest][Forest][Forest][Forest][Cabin ][Forest][Forest][Forest][Forest][Forest][Forest]
    [Forest][Forest][Forest][Forest][Path  ][Forest][Forest][Forest][Forest][Forest][Forest]
    [Forest][Forest][Forest][Forest][Path  ][Forest][Forest][Forest][Forest][Forest][Forest]
    [Forest][Forest][Forest][Forest][Path  ][Forest][Forest][Forest][Forest][Forest][Forest]
    [Forest][Forest][Forest][Forest][Start ][Forest][Forest][Forest][Forest][Forest][Forest]
              ↓
          SOUTH (Spawn Point at 10,5)
```

### Coordinate System
- Origin (0,0): Northwest corner
- Start Position (10, 5): Southern center
- Cabin Position (6, 5): North of start
- Lake: Central area (rows 1-5, cols 1-9)

### Four Seasonal Biomes

#### 1. WEST - Desert/Summer Biome
- **Climate:** Hot, dry, intense sun
- **Terrain:** Sandy dunes, cacti, oasis near lake
- **Flora:** Cacti, succulents, desert flowers, palm trees at oasis
- **Fauna:** Lizards, desert foxes, scorpions, hawks
- **Lake Edge:** Oasis with date palms, cool water refuge

#### 2. NORTH - Spring/Autumn Biome
- **Climate:** Mild, comfortable, occasional breeze
- **Terrain:** Deciduous forest, meadows
- **Vista:** Distant 8000m+ snow-capped mountains visible
- **Special:** Sunrise illuminates the distant peaks - spectacular view
- **Flora:** Maples, oaks, wildflowers, mushrooms
- **Fauna:** Deer, rabbits, squirrels, songbirds

#### 3. EAST - Winter/Snow Biome
- **Climate:** Cold, snowy, crisp air
- **Terrain:** Snow-covered evergreen forest
- **Special:** Aurora borealis visible from evening onwards
- **Flora:** Pine trees, frozen berries, icicles
- **Fauna:** Snow foxes, owls, wolves (distant howls), caribou

#### 4. SOUTH - Starting Area
- **Climate:** Neutral, transitional
- **Terrain:** Mixed forest, well-worn path leading north
- **Flora:** Mixed vegetation
- **Fauna:** Common woodland creatures

---

## The Cabin (Detail Zone)

### Cabin Exterior
- Wooden construction, weathered but sturdy
- Chimney (smoke visible when fire is lit)
- Front door (OPENABLE/CLOSEABLE - persistent state)

### Cabin Interior - Main Room
- **Fireplace:** Stone hearth, requires fuel to light
  - States: COLD, SMOLDERING, BURNING, ROARING
  - Affects room temperature
- **Matchbox:** On mantelpiece above fireplace
- **Kindling:** Small bundle for fire starting
- **Curious Items:** Mysterious artifacts, old books, strange instruments
- **Furniture:** Wooden table, chairs, old rug

### Cabin Interior - Terrace/Back Porch
- Overlooks the lake
- Three viewing directions: West (Desert/Oasis), North (Lake center), East (Snowy shore)
- Dynamic wildlife observation

### Wood Shed (Northwest of Cabin)
- **Woodpile:** Log count (starts: 10-15 logs)
- **Chopping Block:** For splitting wood
- **Axe:** On ground, can be picked up
- **Log Stack:** Unsplit logs (starts: 5-8)

---

## Character System

### Stats
```rust
struct Character {
    health: f32,          // 0-100
    warmth: f32,          // 0-100 (50 = comfortable)
    energy: f32,          // 0-100
    mood: f32,            // 0-100

    // Position
    x: i32,
    y: i32,
    facing: Direction,
    current_room: Option<Room>,  // Interior locations
}
```

### Skills
```rust
struct Skills {
    woodcutting: u8,      // 1-100, affects chop success rate
    fire_making: u8,      // 1-100, affects fire starting
    observation: u8,      // 1-100, affects wildlife spotting detail
    foraging: u8,         // 1-100, affects finding items
}
```

### Inventory
- Weight-limited carrying capacity
- Items have properties: weight, warmth_value, fuel_value, etc.

---

## Interaction System

### Movement Commands
- `move <direction>` - Move N/S/E/W
- `look` - Describe current location
- `look <direction>` - Look in a direction without moving
- `enter <location>` - Enter building/area
- `exit` - Leave current interior

### Object Interactions
- `examine <object>` - Detailed description
- `take <object>` - Add to inventory
- `drop <object>` - Remove from inventory
- `use <object>` - Context-dependent action
- `use <object> on <target>` - Combined action
- `open <object>` - Open doors, containers
- `close <object>` - Close doors, containers

### Cabin-Specific Actions
- `chop <log>` - Attempt to chop wood (skill check)
- `light <object>` - Use match to light kindling/fire
- `stoke <fire>` - Add fuel to existing fire
- `rest` - Recover energy
- `observe <direction>` - Watch wildlife from terrace

---

## Simulation Systems

### Time System
- Each action advances time slightly
- Time of day affects: lighting, wildlife activity, aurora visibility, temperature
- Cycles: Dawn → Morning → Noon → Afternoon → Dusk → Evening → Night → Midnight

### Temperature System
```rust
fn calculate_temperature(biome: Biome, time: TimeOfDay, indoor: bool, fire_state: FireState) -> f32 {
    // Base temps by biome
    // Modified by time of day
    // Indoor modifier
    // Fire heat contribution
}
```

### Weather System (per biome)
- Desert: Clear, Sandstorm, Heat Wave
- Spring/Autumn: Clear, Cloudy, Light Rain, Fog
- Winter: Clear, Snowing, Blizzard
- Affects visibility, temperature, wildlife behavior

### Wildlife Simulation
Each creature has:
```rust
struct Wildlife {
    species: Species,
    position: (i32, i32),
    behavior: Behavior,  // GRAZING, SLEEPING, MOVING, ALERT
    schedule: ActivitySchedule,  // Diurnal, nocturnal, crepuscular
}
```

Wildlife spawns in appropriate biomes and moves between cells.

---

## Fire-Making Detailed Flow

### Required Items
1. **Axe** (from wood shed floor)
2. **Log** (from log stack)
3. **Chopping Block** (fixed location)
4. **Kindling** (from cabin or chopping success byproduct)
5. **Matchbox** (from fireplace mantel)
6. **Firewood** (result of chopping)

### Process
1. `take axe` - Pick up axe from wood shed
2. `take log` - Get unsplit log from stack
3. `use log on chopping_block` - Place log on block
4. `use axe on log` - Attempt to chop
   - Roll: `base_success(50%) + skill_bonus(woodcutting/2)`
   - Success: Creates 2-4 firewood pieces
   - Failure: Minor energy loss, possible small damage
5. `take firewood` - Collect split wood
6. Return to cabin main room
7. `use firewood on fireplace` - Add fuel
8. `take matchbox` - Get matches
9. `take kindling` - Get kindling
10. `use match on kindling` - Light kindling
11. `use kindling on fireplace` - Add lit kindling to fuel
    - Roll: `base_success(40%) + fire_making_skill/2`
    - Success: Fire begins!
    - Failure: Kindling consumed, try again

### Fire States
- **COLD:** No fire, no warmth
- **SMOLDERING:** Just started, minimal warmth, can die
- **BURNING:** Steady fire, good warmth
- **ROARING:** Maximum heat, consumes fuel faster

---

## MCP Server Interface

### Tools Exposed

```json
{
  "tools": [
    {
      "name": "look",
      "description": "Observe your surroundings or a specific direction",
      "parameters": {
        "direction": "optional: north/south/east/west/up/down"
      }
    },
    {
      "name": "move",
      "description": "Move in a direction",
      "parameters": {
        "direction": "required: north/south/east/west"
      }
    },
    {
      "name": "examine",
      "description": "Closely examine an object or creature",
      "parameters": {
        "target": "required: name of object to examine"
      }
    },
    {
      "name": "take",
      "description": "Pick up an item",
      "parameters": {
        "item": "required: item name"
      }
    },
    {
      "name": "drop",
      "description": "Drop an item from inventory",
      "parameters": {
        "item": "required: item name"
      }
    },
    {
      "name": "use",
      "description": "Use an item, optionally on a target",
      "parameters": {
        "item": "required: item to use",
        "target": "optional: target to use item on"
      }
    },
    {
      "name": "open",
      "description": "Open a door or container",
      "parameters": {
        "target": "required: what to open"
      }
    },
    {
      "name": "close",
      "description": "Close a door or container",
      "parameters": {
        "target": "required: what to close"
      }
    },
    {
      "name": "inventory",
      "description": "List items you are carrying",
      "parameters": {}
    },
    {
      "name": "status",
      "description": "Check your current condition",
      "parameters": {}
    },
    {
      "name": "wait",
      "description": "Pass time, observing your surroundings",
      "parameters": {
        "duration": "optional: short/medium/long"
      }
    },
    {
      "name": "simulate",
      "description": "Advance world simulation by specified ticks",
      "parameters": {
        "ticks": "optional: number of simulation steps (default 1)"
      }
    }
  ]
}
```

---

## Persistence

### State File: `world_state.json`
```json
{
  "version": "1.0",
  "time": {
    "day": 1,
    "time_of_day": "morning",
    "ticks": 0
  },
  "player": {
    "x": 10,
    "y": 5,
    "room": null,
    "health": 100,
    "warmth": 50,
    "energy": 100,
    "mood": 70,
    "skills": {
      "woodcutting": 10,
      "fire_making": 10,
      "observation": 10,
      "foraging": 10
    },
    "inventory": []
  },
  "world": {
    "cabin": {
      "door_open": false,
      "fire_state": "cold",
      "fire_fuel": 0,
      "items": ["matchbox", "kindling", "old_book", "strange_compass"]
    },
    "wood_shed": {
      "logs": 6,
      "firewood": 0,
      "axe_location": "floor"
    }
  },
  "wildlife": [
    {"species": "deer", "x": 2, "y": 3, "behavior": "grazing"},
    {"species": "rabbit", "x": 8, "y": 7, "behavior": "foraging"}
  ],
  "weather": {
    "west": "clear",
    "north": "cloudy",
    "east": "light_snow"
  }
}
```

---

## Description Writing Style

### Atmosphere
- Rich sensory details (sight, sound, smell, touch, temperature)
- Peaceful, contemplative tone
- Seasonal and time-appropriate details
- Wildlife behaves naturally, not just "exists"

### Example Description (Morning at Lake, North Shore)
```
The morning mist rises gently from the lake's mirror-still surface.
In the distance, the snow-capped peaks catch the first golden rays of
sunrise, their white crowns glowing pink and orange against the pale
blue sky.

A family of ducks glides silently near the reeds, their wake creating
perfect V-shaped ripples. The air smells of damp earth and pine
needles. Somewhere in the forest behind you, a woodpecker begins its
rhythmic tapping.

The water is cool but not cold—perfect autumn temperature. A fish
breaks the surface briefly, creating concentric circles that slowly
expand and fade.
```

---

## Technical Architecture

```
rubber-duck-mcp/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, MCP server setup
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── server.rs     # MCP protocol implementation
│   │   └── tools.rs      # Tool definitions
│   ├── world/
│   │   ├── mod.rs
│   │   ├── map.rs        # World grid, tiles, biomes
│   │   ├── time.rs       # Time and weather systems
│   │   └── simulation.rs # Wildlife, state updates
│   ├── entity/
│   │   ├── mod.rs
│   │   ├── player.rs     # Player state, inventory
│   │   ├── wildlife.rs   # Animal behaviors
│   │   └── objects.rs    # Interactive objects
│   ├── actions/
│   │   ├── mod.rs
│   │   ├── movement.rs   # Navigation
│   │   ├── interaction.rs # Object manipulation
│   │   └── crafting.rs   # Fire-making, etc.
│   ├── descriptions/
│   │   ├── mod.rs
│   │   └── generator.rs  # Rich text generation
│   └── persistence/
│       ├── mod.rs
│       └── state.rs      # Save/load JSON state
├── data/
│   └── initial_state.json
└── docs/
    └── GDD.md
```

---

## Future Expansion Ideas

- Fishing system
- Cooking over fire
- More cabin rooms (bedroom, cellar)
- Boat to explore lake
- Cave system on eastern mountain
- Seasonal events (meteor shower, animal migration)
- Photography/journaling mechanic
- Weather-dependent activities (rain meditation, snow angels)

---

## Development Priorities

### Phase 1: Core
1. World map and biome system
2. Basic movement and look
3. MCP server interface
4. Persistence

### Phase 2: Cabin
1. Cabin interior zones
2. Door open/close mechanics
3. Basic item interaction

### Phase 3: Fire System
1. Wood chopping mechanics
2. Fire-building process
3. Temperature effects

### Phase 4: Simulation
1. Time progression
2. Wildlife behaviors
3. Weather system

### Phase 5: Polish
1. Rich descriptions
2. Skill system effects
3. Mood and comfort mechanics
