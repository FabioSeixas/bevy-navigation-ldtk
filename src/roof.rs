use bevy::prelude::*;
use bevy_ecs_tilemap::{
    map::TilemapId,
    tiles::{TileColor, TilePos},
};

use crate::{
    agent::Agent,
    spatial_idx::SpatialIndex,
    world::{GridPosition, TileType},
};

pub struct RoofPlugin;

impl Plugin for RoofPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, roof_opacity_system);
    }
}

/// Marker component for tiles that should be transparent when an agent is under them.
#[derive(Component)]
pub struct Roof;

fn roof_opacity_system(
    agents_q: Query<&GridPosition, With<Agent>>,
    mut tiles_q: Query<(&TilemapId, &TilePos, &mut TileColor)>,
    spatial_idx: Res<SpatialIndex>,
) {
    const RADIUS: i32 = 4;
    const TRANSPARENT_ALPHA: f32 = 0.1;
    const OPAQUE_ALPHA: f32 = 1.0;

    // 1. Find all unique positions around "inside" agents that should be transparent.
    let mut transparent_positions = std::collections::HashSet::new();
    for agent_pos in &agents_q {
        if let Some(tile_data) = spatial_idx.get_entity_data(agent_pos.x, agent_pos.y) {
            if matches!(tile_data.tile_type, TileType::Inside | TileType::Door) {
                // Agent is inside, so mark a radius of tiles for transparency.
                for x in (agent_pos.x.saturating_sub(RADIUS))..=(agent_pos.x + RADIUS) {
                    for y in (agent_pos.y.saturating_sub(RADIUS))..=(agent_pos.y + RADIUS) {
                        transparent_positions.insert((x, y));
                    }
                }
            }
        }
    }

    // 2. Update opacity for all tiles based on their layer and if they are a roof.
    for (tilemap_id, tile_pos, mut tile_color) in &mut tiles_q {
        let pos = (tile_pos.x as i32, tile_pos.y as i32);

        if let Some(tile_data) = spatial_idx.get_entity_data(pos.0, pos.1) {
            // Check if the tile is a roof-like tile
            if matches!(tile_data.tile_type, TileType::Inside | TileType::Door) {
                // And check if it's on the correct layer for that logical tile
                if Some(tilemap_id.0) == tile_data.tilemap_entity {
                    let target_alpha = if transparent_positions.contains(&pos) {
                        TRANSPARENT_ALPHA
                    } else {
                        OPAQUE_ALPHA
                    };

                    if tile_color.0.alpha() != target_alpha {
                        tile_color.0.set_alpha(target_alpha);
                    }
                }
            }
        }
    }
}
