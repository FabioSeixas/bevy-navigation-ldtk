use bevy::{platform::collections::HashMap, prelude::*};

use crate::world::components::*;

#[derive(Clone, Copy, Debug)]
pub struct TileData {
    pub entity: Entity,
    pub flags: TileFlags,
    pub tilemap_entity: Option<Entity>,
}

impl TileData {
    pub fn is_building(&self) -> bool {
        self.flags
            .intersects(TileFlags::INSIDE | TileFlags::DOOR | TileFlags::WALL)
    }

    pub fn is_wall(&self) -> bool {
        self.flags.contains(TileFlags::WALL)
    }

    pub fn is_outside(&self) -> bool {
        self.flags.contains(TileFlags::OUTSIDE)
    }

    pub fn is_valid_destination(&self) -> bool {
        if self.flags.contains(TileFlags::DOOR) {
            return false;
        }

        self.is_walkable()
    }

    pub fn is_indoor(&self) -> bool {
        self.flags.intersects(TileFlags::DOOR | TileFlags::INSIDE)
    }

    pub fn is_walkable(&self) -> bool {
        // Walls are never walkable
        if self.flags.contains(TileFlags::WALL) {
            return false;
        }

        // Furniture blocks movement
        if self.flags.contains(TileFlags::FURNITURE) {
            return false;
        }

        // Must be some form of traversable terrain
        if !self.flags.contains(TileFlags::TRAVERSABLE_TERRAIN) {
            return false;
        }

        true
    }

    pub fn is_traversable_to(&self, destination_tile: &TileData) -> bool {
        if !destination_tile.is_walkable() {
            return false;
        }

        let current_is_inside = self.flags.contains(TileFlags::INSIDE);
        let current_is_outside = self.flags.contains(TileFlags::OUTSIDE);
        let current_is_door = self.flags.contains(TileFlags::DOOR);

        let dest_is_inside = destination_tile.flags.contains(TileFlags::INSIDE);
        let dest_is_outside = destination_tile.flags.contains(TileFlags::OUTSIDE);
        let dest_is_door = destination_tile.flags.contains(TileFlags::DOOR);

        // Allow movement between any two 'outside' tiles
        if current_is_outside && dest_is_outside {
            return true;
        }

        // Allow movement between any two 'inside' tiles
        if current_is_inside && dest_is_inside {
            return true;
        }

        // Allow movement to/from a door
        if current_is_door || dest_is_door {
            return true;
        }

        false
    }
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
