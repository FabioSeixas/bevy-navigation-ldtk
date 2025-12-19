use bevy::prelude::*;

use crate::world::{
    spatial_idx::SpatialIndex,
    systems::{on_add_tile, on_add_tile_enum_tags},
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialIndex>()
            .add_observer(on_add_tile_enum_tags)
            .add_observer(on_add_tile);
    }
}
