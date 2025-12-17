use bevy::{platform::collections::HashMap, prelude::*};

use crate::world::{GridPosition, TileType};

#[derive(Clone, Copy, Debug)]
pub struct TileData {
    pub entity: Entity,
    pub tile_type: TileType,
}

#[derive(Resource, Default, Debug)]
pub struct SpatialIndex {
    pub map: HashMap<(i32, i32), TileData>,
}

impl SpatialIndex {
    pub fn get_nearby(&self, origin_x: i32, origin_y: i32) -> Vec<(TileData, GridPosition)> {
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

    pub fn get_entity_data(&self, x: i32, y: i32) -> Option<TileData> {
        // println!("get_entity: {} {}", x, y);
        match self.map.get(&(x, y)) {
            Some(data) => Some(*data),
            None => None,
        }
    }

    pub fn get_entity(&self, x: i32, y: i32) -> Option<Entity> {
        // println!("get_entity: {} {}", x, y);
        match self.map.get(&(x, y)) {
            Some(data) => Some(data.entity),
            None => None,
        }
    }
}
