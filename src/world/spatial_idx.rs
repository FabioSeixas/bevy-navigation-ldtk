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
            .intersects(TileFlags::INSIDE | TileFlags::DOOR | TileFlags::WALL | TileFlags::ROOF)
    }

    pub fn is_wall(&self) -> bool {
        self.flags.contains(TileFlags::WALL)
    }

    pub fn is_roof(&self) -> bool {
        self.flags.contains(TileFlags::ROOF)
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
    pub fn get_tile_data(&self, x: i32, y: i32) -> Option<TileData> {
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
