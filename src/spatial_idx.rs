use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::constants::TILE_SIZE;

#[derive(Resource, Default, Debug)]
pub struct SpatialIndex {
    // map: HashMap<(i32, i32), HashSet<Entity>>,
    map: HashMap<(i32, i32), Entity>,
}

#[derive(Component)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

impl SpatialIndex {
    // Lookup all entities within adjacent cells of our spatial index
    pub fn get_nearby_from_vec(&self, pos: Vec2) -> Vec<Entity> {
        let tile = (
            (pos.x / TILE_SIZE).floor() as i32,
            (pos.y / TILE_SIZE).floor() as i32,
        );
        let mut nearby = Vec::new();
        for x in -1..2 {
            for y in -1..2 {
                if let Some(tile_entity) = self.map.get(&(tile.0 + x, tile.1 + y)) {
                    // nearby.extend(mines.iter());
                    nearby.push(*tile_entity);
                }
            }
        }
        nearby
    }

    pub fn get_nearby(&self, origin_x: i32, origin_y: i32) -> Vec<Entity> {
        let mut nearby = Vec::new();
        for x in -1..2 {
            for y in -1..2 {
                if let Some(tile_entity) = self.map.get(&(origin_x + x, origin_y + y)) {
                    // nearby.extend(mines.iter());
                    nearby.push(*tile_entity);
                }
            }
        }

        println!("get_nearby: {:?}", nearby);
        nearby
    }

    pub fn get_entity(&self, x: i32, y: i32) -> Entity {
        self.map.get(&(x, y)).unwrap().clone()
    }
}

pub fn on_add_tile(add: On<Add, Tile>, query: Query<&Tile>, mut index: ResMut<SpatialIndex>) {
    let tile = query.get(add.entity).unwrap();
    index.map.entry((tile.x, tile.y)).or_insert(add.entity);
}
