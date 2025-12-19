use bevy::prelude::*;

use crate::world::{
    spatial_idx::SpatialIndex,
    systems::{on_add_tile, on_add_tile_enum_tags, on_agent_entered_tile, on_agent_left_tile},
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialIndex>()
            .add_observer(on_add_tile_enum_tags)
            .add_observer(on_add_tile)
            .add_observer(on_agent_left_tile)
            .add_observer(on_agent_entered_tile);
    }
}
