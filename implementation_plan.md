# Opacity for Roof Components

This query at `roof.rs` is wrong:

    mut roofs_q: Query<(&GridPosition, &mut TileColor), With<Roof>>,

TileColor is from `bevy_ecs_tilemap`. It wont exist and Entity with GridPosition and TileColor.
The entities with GridPosition are defined by us, the entities with TileColor are default inserted by `bevy_ecs_tilemap`.

So, the correct logic is to find all the points where opacity must be set. Create a hasmap for this.

Search over all entities that has TileColor + TilePos (tile position) Components, 
If the entity has a tile position matching one of our hasmap entries, then set the alpha of it.

I like the idea of set alpha default to 1.0 to all first. 
Then, find the ones that need a different value and update the hashmap.
Then, go to `bevy_ecs_tilemap` entities, and set the value of each one.
