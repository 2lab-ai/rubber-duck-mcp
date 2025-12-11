# Rubber Duck Survival: Realism Design Document (Blueprint System)

## 1. Interaction Philosophy: "Simple Input, Complex Outcome"
To maintain CLI usability while ensuring depth, all interactions follow the `use [Tool/Material] on [Target]` pattern.

### 1.1. The "Use" Logic
- **Gathering:** `use [Hands/Tool] on [Resource]`
    - *Example:* `use hands on bush` -> Get `stick`, `berries`. (Skill: Foraging)
    - *Example:* `use knife on tree` -> Get `bark`, `sap`. (Skill: Survival)
- **Processing:** `use [Tool] on [Item]`
    - *Example:* `use knife on log` -> Get `kindling`. (Skill: Woodworking)
    - *Example:* `use stone on stone` -> Get `sharp_stone`. (Skill: Stonemasonry)
- **Assembly:** `use [Material] on [Blueprint]`
    - *See Section 2: Blueprint System.*

## 2. The Blueprint (Project) System
Complex items require planning and incremental assembly.

### 2.1. Workflow
1.  **Initiate:** Player creates a blueprint.
    - Command: `create [item_name]` (e.g., `create campfire`, `create stone_axe`)
    - Result: A `[Blueprint: Item Name]` object appears.
2.  **Inspect:** Player checks requirements.
    - Command: `examine blueprint`
    - Output: "Requires: Logs (0/3), Tinder (0/1), Kindling (0/2)."
3.  **Build:** Player adds materials.
    - Command: `use log on blueprint`
    - Effect: Item removed from inventory, Blueprint progress updated. **Time passes (Simulation ticks).**
4.  **Finalize:** When requirements are met, the Blueprint transforms into the final Object/Item.

### 2.2. Recipe Examples (Basic Survival Cycle)

**[Blueprint: Stone Knife]**
- **Skill:** Survival Lv.1
- **Inputs:**
    - `Sharp Stone` x1 (Blade)
    - `Stick` x1 (Handle)
    - `Plant Fiber` x1 (Binding)
- **Process Time:** 10 mins per ingredient.

**[Blueprint: Campfire]**
- **Skill:** Survival Lv.1
- **Inputs:**
    - `Stones` x4 (Ring)
    - `Tinder` x1 (Starter)
    - `Kindling` x3 (Base)
    - `Logs` x2 (Fuel)
- **Process Time:** 20 mins total.

## 3. Skill System
Every action is tied to a skill. Skills determine success rates, processing time, and durability loss.

### 3.1. Skill List
- **Foraging:** Finding items in the wild (`use hands on ...`).
- **Woodworking:** Chopping, whittling, planning wood recipes.
- **Stonemasonry:** Knapping stones, building stone structures.
- **Survival:** Fire starting, shelter building, weather resistance.
- **Tailoring:** Processing hides, sewing, creating cordage.
- **Cooking:** Preparing food, boiling water.

### 3.2. Progression
- **XP Gain:** Successful `use` actions grant XP.
- **Level Up:** Unlock new Blueprints (e.g., Woodworking Lv.3 unlocks `Wooden Shelter`).

## 4. Resource & Time Realism
- **Time Cost:** Every `use` action advances world time (`simulate` ticks).
    - *Example:* `use knife on log` advances time by 10 minutes.
- **Durability:** Tools lose durability with every use. Broken tools disappear or become `Broken [Item]`.
- **Resource Depletion:** Trees/Bushes have a finite supply. `use` depletes them. They regenerate slowly over days.

## 5. Immediate Development Goals (Survival Cycle)
1.  **Harvesting:** `use hands on bush` -> Stick/Fiber. `use stone on stone` -> Sharp Stone.
2.  **Tool Making:** `create stone_knife` -> Assemble.
3.  **Fire Making:** `create campfire` -> Assemble -> `use fire_starter on campfire`.
4.  **Water:** `use muddy_water on fire` (Boil) -> `clean_water`.