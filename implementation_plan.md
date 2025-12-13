# Pathfinding Implementation Plan

This document outlines the plan to implement the "Buildings as Closed Areas with Door Transitions" pathfinding solution.

## Architecture Decisions

### 1. Data Source for Pathfinder

We will use a central spatial index to provide world layout data to the pathfinder. The chosen approach is to **augment the existing `SpatialIndex` resource to also cache the `TileType` for each tile.**

This is preferable to creating a separate `WorldGrid` resource because it consolidates data, reuses existing code, and maintains decoupling between the pathfinder and the Bevy ECS.

### 2. Static vs. Dynamic Obstacles

To handle both static walls and dynamic agents, we will use two separate mechanisms:

*   **`TileType` Enum**: This defines the **static geometry** of the world (`Wall`, `Inside`, `Outside`, `Door`). A tile's type is set once at level load and does not change. This data will be cached in the `SpatialIndex`.
*   **`Occupied` Component**: This marks the **dynamic state** of a tile being temporarily blocked by an agent. It will be added and removed as agents move. It will **no longer be used for walls**.

---

## Revised High-Level Implementation Plan

### Step 1: Update Data Structures (`src/world.rs` and `src/spatial_idx.rs`)

1.  **Add `TileType` Enum (in `src/world.rs`):**
    *   Define a public `enum TileType { Outside, Inside, Wall, Door }`.
    *   Make it a Bevy `Component`.
    *   Provide a `Default` implementation that returns `TileType::Wall`.

2.  **Create `TileData` Struct (in `src/spatial_idx.rs`):**
    *   Define a new public struct: `pub struct TileData { pub entity: Entity, pub tile_type: TileType }`.

3.  **Modify `SpatialIndex` (in `src/spatial_idx.rs`):**
    *   Change the `map` field's type to `HashMap<(i32, i32), TileData>`.

### Step 2: Update Data Population Logic (`src/world.rs`)

1.  **Modify `on_add_tile` and `on_add_tile_enum_tags`:**
    *   `on_add_tile`: When a tile entity is created, insert it into the `SpatialIndex` with a default `TileType::Wall`.
    *   `on_add_tile_enum_tags`: This system will now *update* the `tile_type` field for the corresponding entry in the `SpatialIndex`. It will **no longer add the `Occupied` component** to walls.

### Step 3: Update Pathfinder (`src/pathfinder.rs` and `src/main.rs`)

1.  **Update `Pathfinder::step` Function Signature:**
    *   Change its signature to accept both static and dynamic world data: `pub fn step(&mut self, spatial_index: &SpatialIndex, dynamic_occupied_tiles: &HashSet<GridPosition>)`.

2.  **Update `check_agent_pathfinding` System (in `main.rs`):**
    *   This system will become responsible for providing all necessary data to the pathfinder.
    *   Each frame, it will query for all entities with an `Occupied` component and create a `HashSet<GridPosition>` of currently blocked tiles.
    *   It will then call `pathfinder.step()` with both the `spatial_idx` resource and this new `HashSet`.

3.  **Implement New Movement Rules in `Pathfinder::step`:**
    *   Inside the `step` function, when checking a neighbor tile, it will now perform two checks:
        *   **Static Check:** Use the `spatial_index` to get the `TileType` for the current and neighbor nodes and apply the `match` statement to enforce geometric rules (walls, doors, etc.).
        *   **Dynamic Check:** Check if the neighbor's `GridPosition` is present in the `dynamic_occupied_tiles` `HashSet`.
    *   A move is only valid if it passes **both** checks.

---
This plan is ready for implementation.
