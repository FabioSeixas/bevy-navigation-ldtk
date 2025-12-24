use bevy::prelude::*;
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct TileFlags: u32 {
        const TRAVERSABLE_TERRAIN = 1 << 0;
        const OUTSIDE             = 1 << 1;
        const INSIDE              = 1 << 2;
        const WALL                = 1 << 3;
        const DOOR                = 1 << 4;
        const FURNITURE           = 1 << 5;
        const ROOF                = 1 << 6;
    }
}

// #########################
// TILE
// #########################
#[derive(Component, Debug)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

// #########################
// GRID POSITION
// #########################
#[derive(Component, Debug, PartialEq, Eq, Clone, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    // Euclidian distance
    pub fn calc_distance(&self, reference: &GridPosition) -> f32 {
        let dx = (reference.x - self.x) as f32;
        let dy = (reference.y - self.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    // TODO: implement 
    pub fn is_adjacent(&self, reference: &GridPosition) -> bool {
        true
    }
}

// #########################
// OCCUPIED
// #########################
#[derive(Component)]
pub struct Occupied;
