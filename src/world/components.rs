use bevy::prelude::*;

// #########################
// TILE
// #########################
#[derive(Component, Debug)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

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

// #########################
// GRID POSITION
// #########################
#[derive(Component, Debug, PartialEq, Eq, Clone, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

// #########################
// OCCUPIED
// #########################
#[derive(Component)]
pub struct Occupied;

// #########################
// ROOF
// Marker component for tiles that should be transparent when an agent is under them.
// #########################
#[derive(Component)]
pub struct Roof;
