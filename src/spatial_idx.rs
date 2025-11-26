use bevy::{platform::collections::HashMap, prelude::*};

use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{GridPosition, Occupied};

#[derive(Resource, Default, Debug)]
pub struct SpatialIndex {
    map: HashMap<(i32, i32), Entity>,
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
            None => None,
        }
    }
}

pub fn on_add_tile(
    add: On<Add, TilemapId>,
    query: Query<&GridCoords>,
    mut index: ResMut<SpatialIndex>,
) {
    let coords = query.get(add.entity).unwrap();
    index.map.entry((coords.x, coords.y)).or_insert(add.entity);
}

pub fn on_add_tile_enum_tags(
    add: On<Add, TileEnumTags>,
    query: Query<&TileEnumTags, (With<GridCoords>, With<TilemapId>)>,
    mut commands: Commands,
) {
    let enum_tags = query.get(add.entity).unwrap();

    if enum_tags.tags.iter().any(|x| x.eq("A")) {
        commands.entity(add.entity).insert(Occupied);
    }
}
