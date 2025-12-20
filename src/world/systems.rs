use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use bevy_ecs_ldtk::prelude::*;

use crate::{
    constants::{GRID_HEIGHT, GRID_WIDTH},
    events::{AgentEnteredTile, AgentLeftTile},
    world::{components::*, grid::Grid, spatial_idx::*},
};

pub fn on_agent_left_tile(event: On<AgentLeftTile>, mut commands: Commands) {
    commands.entity(event.entity).remove::<Occupied>();
}

pub fn on_agent_entered_tile(event: On<AgentEnteredTile>, mut commands: Commands) {
    commands.entity(event.entity).insert(Occupied);
}

pub fn spawn_grid(mut commands: Commands) {
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let pos = Grid::grid_to_world(x, y);

            commands.spawn((
                Transform::from_translation(pos),
                Tile { x, y },
                GridPosition { x, y },
            ));
        }
    }
}

pub fn on_add_tile(add: On<Add, Tile>, query: Query<&Tile>, mut index: ResMut<SpatialIndex>) {
    if let Ok(tile) = query.get(add.entity) {
        index.map.entry((tile.x, tile.y)).or_insert(TileData {
            entity: add.entity,
            flags: TileFlags::TRAVERSABLE_TERRAIN,
            tilemap_entity: None,
        });
    }
}

pub fn on_add_tile_enum_tags(
    add: On<Add, TileEnumTags>,
    query_third_party_tile: Query<(&TileEnumTags, &GridCoords, &TilemapId)>,
    mut index: ResMut<SpatialIndex>,
) {
    let (enum_tags, coords, tilemap_id) = query_third_party_tile.get(add.entity).unwrap();

    let mut tile_flags = TileFlags::empty();

    if enum_tags.tags.iter().any(|t| t == "Wall") {
        tile_flags |= TileFlags::WALL;
    } else if enum_tags.tags.iter().any(|t| t == "Door") {
        tile_flags |= TileFlags::DOOR;
    } else if enum_tags.tags.iter().any(|t| t == "Inside") {
        tile_flags |= TileFlags::INSIDE;
    } else if enum_tags.tags.iter().any(|t| t == "Outside") {
        tile_flags |= TileFlags::OUTSIDE;
    } else if enum_tags.tags.iter().any(|t| t == "Furniture") {
        tile_flags |= TileFlags::FURNITURE;
    } else if enum_tags.tags.iter().any(|t| t == "Roof") {
        tile_flags |= TileFlags::ROOF;
    } else {
        // Don't change type if no matching tag found
        return;
    };

    if let Some(tile_data) = index.map.get_mut(&(coords.x, coords.y)) {
        tile_data.flags |= tile_flags;
        tile_data.tilemap_entity = Some(tilemap_id.0);
    }
}
