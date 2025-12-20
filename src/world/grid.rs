use bevy::prelude::*;

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

    pub fn get_random_position() -> GridPosition {
        let mut rnd = rand::thread_rng();
        GridPosition {
            x: rnd.gen_range(0..GRID_WIDTH),
            y: rnd.gen_range(0..GRID_HEIGHT),
        }
    }
}
