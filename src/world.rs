use bevy::prelude::*;

use bevy_ecs_ldtk::prelude::*;
use rand::Rng;

use crate::{
    constants::{GRID_HEIGHT, GRID_WIDTH, TILE_SIZE},
    spatial_idx::SpatialIndex,
};

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
        index.map.entry((tile.x, tile.y)).or_insert(add.entity);
    }
}

pub fn on_add_tile_enum_tags(
    add: On<Add, TileEnumTags>,
    query_third_party_tile: Query<(&TileEnumTags, &GridCoords)>,
    query_main_tile: Query<(Entity, &Tile)>,
    mut commands: Commands,
) {
    let (enum_tags, coords) = query_third_party_tile.get(add.entity).unwrap();

    dbg!(enum_tags);
    if enum_tags.tags.iter().any(|x| x.eq("A")) {
        for (entity, tile) in query_main_tile {
            if tile.x == coords.x && tile.y == coords.y {
                println!("add Occupied");
                commands.entity(entity).insert(Occupied);
            }
        }
    }
}
