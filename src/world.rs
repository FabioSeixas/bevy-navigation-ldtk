use bevy::prelude::*;

use bevy_ecs_ldtk::prelude::*;
use rand::Rng;

use crate::{
    constants::{GRID_HEIGHT, GRID_WIDTH, TILE_SIZE},
    roof::Roof,
    spatial_idx::{SpatialIndex, TileData},
};

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Outside,
    Inside,
    Wall,
    Door,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Wall
    }
}

#[derive(Component, Debug)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

pub struct Grid;

impl Grid {
    /// Convert grid coordinates → world coordinates (Vec3)
    pub fn grid_to_world(x: i32, y: i32) -> Vec3 {
        Vec3::new(
            x as f32 * TILE_SIZE + (TILE_SIZE / 2.),
            y as f32 * TILE_SIZE + (TILE_SIZE / 2.),
            0.0,
        )
    }

    pub fn world_to_grid(pos: Vec2) -> GridPosition {
        GridPosition {
            x: (pos.x / TILE_SIZE) as i32,
            y: (pos.y / TILE_SIZE) as i32,
        }
    }

    pub fn get_random_position() -> GridPosition {
        let mut rnd = rand::thread_rng();
        GridPosition {
            x: rnd.gen_range(0..GRID_WIDTH),
            y: rnd.gen_range(0..GRID_HEIGHT),
        }
    }

    pub fn coords_to_grid_position(c: GridCoords) -> GridPosition {
        GridPosition { x: c.x, y: c.y }
    }

    pub fn grid_position_to_coords(gp: GridPosition) -> GridCoords {
        GridCoords { x: gp.x, y: gp.y }
    }
}

/// Position on the grid
#[derive(Component, Debug, PartialEq, Eq, Clone, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Occupied;

pub fn on_add_tile(
    add: On<Add, Tile>,
    query: Query<&Tile>,
    mut index: ResMut<SpatialIndex>,
) {
    if let Ok(tile) = query.get(add.entity) {
        index.map.entry((tile.x, tile.y)).or_insert(TileData {
            entity: add.entity,
            tile_type: TileType::default(),
        });
    }
}

pub fn on_add_tile_enum_tags(
    add: On<Add, TileEnumTags>,
    query_third_party_tile: Query<(&TileEnumTags, &GridCoords)>,
    mut index: ResMut<SpatialIndex>,
    mut commands: Commands,
) {
    let (enum_tags, coords) = query_third_party_tile.get(add.entity).unwrap();

    let tile_type = if enum_tags.tags.iter().any(|t| t == "Wall") {
        TileType::Wall
    } else if enum_tags.tags.iter().any(|t| t == "Door") {
        TileType::Door
    } else if enum_tags.tags.iter().any(|t| t == "Inside") {
        commands.entity(add.entity).insert(Roof);
        TileType::Inside
    } else if enum_tags.tags.iter().any(|t| t == "Outside") {
        TileType::Outside
    } else {
        // Don't change type if no matching tag found
        return;
    };

    if let Some(tile_data) = index.map.get_mut(&(coords.x, coords.y)) {
        tile_data.tile_type = tile_type;
    }
}
