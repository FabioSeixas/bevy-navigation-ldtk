# Refactoring Plan for Better Separation of Concerns

## Introduction

This document outlines a refactoring plan to improve the separation of concerns within the codebase. The goal is to create a more modular, maintainable, and extensible architecture by decoupling the `agent`, `pathfinder`, and `world` modules.

## Problem

The current implementation has two main areas where responsibilities are mixed:

1.  **The `agent` module is overloaded.** It manages its own state, drives the pathfinding process, and directly modifies the world's `Occupied` state.
2.  **The `pathfinder` module contains world logic.** It has hardcoded rules about which tiles are traversable, which should be a property of the world itself.

This tight coupling makes the code harder to understand, test, and extend.

## Proposed Refactoring

### Step 1: Centralize World Rules

*   **Why:** The rules for movement and traversability are fundamental properties of the game world, not the pathfinding algorithm. Centralizing this logic in `world.rs` makes the `world` module the single source of truth for the world's "physics." This makes the `Pathfinder` more generic, as it no longer needs to know the specific rules of this world, and it allows other systems to query the same rules if needed.

*   **How:**
    1.  Create a new public function in `src/world.rs`: `pub fn is_traversable(from_tile: &TileData, to_tile: &TileData) -> bool`.
    2.  Move the logic that checks `TileType` from `Pathfinder::step` in `src/pathfinder.rs` into this new `is_traversable` function.
    3.  Modify `Pathfinder::step` to call `world.is_traversable` by passing the relevant tile data. The `Pathfinder` will now depend on the `SpatialIndex` and this new function to determine valid moves.

### Step 2: Introduce a Pathfinding Service

*   **Why:** To decouple the agent's high-level goal ("I need a path") from the low-level process of calculating that path. The agent should not be responsible for managing the state and execution of the pathfinding algorithm. This change will make the system more robust and allow for different pathfinding implementations in the future without changing the agent's code.

*   **How:**
    1.  Create two new components:
        *   `struct PathRequest { pub start: IVec2, pub end: IVec2 }`
        *   `struct PathResult { pub path: Vec<IVec2> }`
    2.  Create a new `pathfinding_system` in a new `src/pathfinding_service.rs` file.
    3.  This system will query for entities with an `Agent` and a `PathRequest` component but no `Pathfinder` component. For each, it will create a `Pathfinder`.
    4.  The `pathfinding_system` will also query for all entities with a `Pathfinder` component and call `step()` on them each frame.
    5.  When a path is found, the `pathfinding_system` will remove the `Pathfinder` and `PathRequest` components and add a `PathResult` component containing the path to the agent entity.
    6.  The `agent`'s `check_agent_pathfinding` system will now be much simpler: if it needs a path, it adds a `PathRequest`. If it sees a `PathResult`, it consumes it and starts walking.

### Step 3: Decouple Agent Movement from World State

*   **Why:** The agent's responsibility is to move; it should not be directly manipulating the world's state (i.e., which tiles are occupied). Using an event-based approach makes the system more modular and easier to debug. It ensures that all world state changes happen in one place, preventing bugs where different systems might try to modify the same state in conflicting ways.

*   **How:**
    1.  Define two new events:
        *   `struct AgentLeftTile(pub Entity);`
        *   `struct AgentEnteredTile(pub Entity);`
    2.  In the `movement_agent` system in `src/agent.rs`, when an agent successfully moves from one tile to another, it will fire these two events with the corresponding tile entities.
    3.  Create a new `world_occupancy_system` in `src/world.rs`.
    4.  This system will listen for `AgentLeftTile` and `AgentEnteredTile` events.
    5.  When it receives `AgentLeftTile`, it will remove the `Occupied` component from the tile entity.
    6.  When it receives `AgentEnteredTile`, it will add the `Occupied` component to the tile entity.
    7.  The `agent` module will no longer have any code that adds or removes the `Occupied` component.

By implementing these three steps, the codebase will have a much cleaner architecture, with each module having a clear and distinct responsibility.
