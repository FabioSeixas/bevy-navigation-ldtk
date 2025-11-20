use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::{GridPosition, constants::TILE_SIZE};

#[derive(Resource, Default, Debug)]
pub struct SpatialIndex {
    // map: HashMap<(i32, i32), HashSet<Entity>>,
    map: HashMap<(i32, i32), Entity>,
}

#[derive(Component, Debug)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

impl SpatialIndex {
    pub fn get_nearby(&self, origin_x: i32, origin_y: i32) -> Vec<(Entity, GridPosition)> {
        let mut nearby = Vec::new();

        // println!("origin: {} {}", origin_x, origin_y);
        for x in -1..2 {
            for y in -1..2 {
                let new_x = origin_x + x;
                let new_y = origin_y + y;

                // println!("curr nearby: {:?}", nearby);
                // println!("new_x: {}", new_x);
                // println!("new_y: {}", new_y);

                // avoid include origin in nearby result
                if new_x == origin_x && new_y == origin_y {
                    continue;
                }

                if let Some(tile_entity) = self.map.get(&(new_x, new_y)) {
                    // println!("added");
                    nearby.push((*tile_entity, GridPosition { x: new_x, y: new_y }));
                    // println!("after add nearby: {:?}", nearby);
                }
            }
        }

        // println!("final nearby: {:?}", nearby);
        nearby
    }

    pub fn get_entity(&self, x: i32, y: i32) -> Option<Entity> {
        // println!("get_entity: {} {}", x, y);
        match self.map.get(&(x, y)) {
            Some(entity) => Some(entity.clone()),
            None => None
        }
    }
}

pub fn on_add_tile(add: On<Add, Tile>, query: Query<&Tile>, mut index: ResMut<SpatialIndex>) {
    let tile = query.get(add.entity).unwrap();
    index.map.entry((tile.x, tile.y)).or_insert(add.entity);
}
