# Pathfinding Issue: Agents Entering Buildings Incorrectly

Problem Summary

Agents in the game move on a grid using pathfinding and occasionally select a random grid position as their destination.

The issue observed was that some of these randomly selected destinations end up being inside buildings, even though there is no valid entrance path to those locations. As a result:

Agents attempt to walk through walls

Pathfinding appears to “cut through” buildings

Interior tiles are treated as globally walkable space

This happens because the pathfinding system only knows about blocked vs walkable tiles, but not about semantic areas like inside and outside.

In short:

> The system does not encode the rule that buildings are closed spaces that can only be entered through doors.

# Chosen Solution

*Buildings as Closed Areas with Door Transitions*

Instead of treating all walkable tiles equally, the map is divided into semantic tile types:

OUTSIDE – normal walkable exterior tiles

INSIDE – interior building tiles

WALL – completely blocked

DOOR – the only valid transition between OUTSIDE and INSIDE

This data is authored in LDTK using Enums and loaded into the game.

# Core Idea

INSIDE tiles are not directly reachable from OUTSIDE tiles.
The only way to transition between the two areas is via a DOOR tile.

# Movement Rules (Conceptual)

WALL → never walkable

OUTSIDE → OUTSIDE → allowed

INSIDE → INSIDE → allowed

OUTSIDE → INSIDE → only if stepping onto DOOR

INSIDE → OUTSIDE → only if stepping onto DOOR

If a transition violates these rules, that edge does not exist in the pathfinding graph.
