use bevy::prelude::*;

use bevy_ecs_ldtk::prelude::*;
use rand::Rng;

use crate::world::components::*;

use crate::constants::{GRID_HEIGHT, GRID_WIDTH, TILE_SIZE};

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
