use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;

use bevy_ecs_ldtk::prelude::*;

use crate::world::{components::*, spatial_idx::*};

pub fn on_add_tile(add: On<Add, Tile>, query: Query<&Tile>, mut index: ResMut<SpatialIndex>) {
    if let Ok(tile) = query.get(add.entity) {
        index.map.entry((tile.x, tile.y)).or_insert(TileData {
            entity: add.entity,
            tile_type: TileType::default(),
            tilemap_entity: None,
        });
    }
}

pub fn on_add_tile_enum_tags(
    add: On<Add, TileEnumTags>,
    query_third_party_tile: Query<(&TileEnumTags, &GridCoords, &TilemapId)>,
    mut index: ResMut<SpatialIndex>,
    mut commands: Commands,
) {
    let (enum_tags, coords, tilemap_id) = query_third_party_tile.get(add.entity).unwrap();

    let mut add_roof = false;

    let tile_type = if enum_tags.tags.iter().any(|t| t == "Wall") {
        TileType::Wall
    } else if enum_tags.tags.iter().any(|t| t == "Door") {
        TileType::Door
    } else if enum_tags.tags.iter().any(|t| t == "Inside") {
        add_roof = true;

        TileType::Inside
    } else if enum_tags.tags.iter().any(|t| t == "Outside") {
        TileType::Outside
    } else {
        // Don't change type if no matching tag found
        return;
    };

    if let Some(tile_data) = index.map.get_mut(&(coords.x, coords.y)) {
        tile_data.tile_type = tile_type;
        tile_data.tilemap_entity = Some(tilemap_id.0);

        if add_roof {
            commands.entity(tile_data.entity).insert(Roof);
        }
    }
}
