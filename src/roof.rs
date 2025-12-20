use bevy::prelude::*;
use bevy_ecs_tilemap::{
    map::TilemapId,
    tiles::{TileColor, TilePos},
};

use crate::{
    agent::Agent,
    world::{components::*, spatial_idx::*},
};

pub struct RoofPlugin;

impl Plugin for RoofPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, roof_opacity_system);
    }
}

// Strategy enum for handling different kinds of roof/interior tiles
enum TransparencyStrategy {
    Roof,
    Indoor,
}

impl TransparencyStrategy {
    fn opacity(&self) -> f32 {
        match self {
            TransparencyStrategy::Roof => 0.6,
            TransparencyStrategy::Indoor => 0.1,
        }
    }

    fn radius(&self) -> i32 {
        match self {
            TransparencyStrategy::Roof => 4,
            TransparencyStrategy::Indoor => 6,
        }
    }

    fn should_affect_neighbor(&self, tile: &TileData) -> bool {
        match self {
            TransparencyStrategy::Roof => tile.is_roof(),
            TransparencyStrategy::Indoor => tile.is_indoor(),
        }
    }

    fn make_adjacent_walls_transparent(&self) -> bool {
        match self {
            TransparencyStrategy::Roof => false,
            TransparencyStrategy::Indoor => true,
        }
    }
}

fn can_be_transparent(tile_data: &TileData) -> Option<TransparencyStrategy> {
    if tile_data.is_roof() {
        return Some(TransparencyStrategy::Roof);
    }
    if tile_data.is_indoor() {
        return Some(TransparencyStrategy::Indoor);
    }
    None
}

fn roof_opacity_system(
    agents_q: Query<&GridPosition, With<Agent>>,
    mut tiles_q: Query<(&TilemapId, &TilePos, &mut TileColor)>,
    spatial_idx: Res<SpatialIndex>,
) {
    const OPAQUE_ALPHA: f32 = 1.0;

    let mut transparent_tiles: std::collections::HashMap<(i32, i32), f32> =
        std::collections::HashMap::new();

    let mut zones_requiring_wall_transparency: std::collections::HashSet<(i32, i32)> =
        std::collections::HashSet::new();

    // 1. Determine base transparency for tiles under roofs/insides based on agent positions.
    for agent_pos in &agents_q {
        if let Some(tile_data) = spatial_idx.get_entity_data(agent_pos.x, agent_pos.y) {
            if let Some(strategy) = can_be_transparent(&tile_data) {
                let opacity = strategy.opacity();
                let radius = strategy.radius();

                for x in (agent_pos.x.saturating_sub(radius))..=(agent_pos.x + radius) {
                    for y in (agent_pos.y.saturating_sub(radius))..=(agent_pos.y + radius) {
                        if let Some(neighbor_tile_data) = spatial_idx.get_entity_data(x, y) {
                            if strategy.should_affect_neighbor(&neighbor_tile_data) {
                                let pos = (x, y);
                                let current_alpha =
                                    transparent_tiles.entry(pos).or_insert(OPAQUE_ALPHA);
                                *current_alpha = current_alpha.min(opacity);

                                if strategy.make_adjacent_walls_transparent() {
                                    zones_requiring_wall_transparency.insert(pos);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Find adjacent walls for zones that require it and set their transparency.
    let inside_opacity = TransparencyStrategy::Indoor.opacity();
    for &(x, y) in &zones_requiring_wall_transparency {
        for nx in (x - 1)..=(x + 1) {
            for ny in (y - 1)..=(y + 1) {
                if nx == x && ny == y {
                    continue;
                }

                if let Some(tile_data) = spatial_idx.get_entity_data(nx, ny) {
                    if tile_data.is_wall() {
                        let wall_pos = (nx, ny);
                        let current_alpha =
                            transparent_tiles.entry(wall_pos).or_insert(OPAQUE_ALPHA);
                        *current_alpha = current_alpha.min(inside_opacity);
                    }
                }
            }
        }
    }

    // 3. Update opacity for all relevant tiles.
    for (tilemap_id, tile_pos, mut tile_color) in &mut tiles_q {
        let pos = (tile_pos.x as i32, tile_pos.y as i32);

        if let Some(tile_data) = spatial_idx.get_entity_data(pos.0, pos.1) {
            if tile_data.is_building() {
                if Some(tilemap_id.0) == tile_data.tilemap_entity {
                    let target_alpha = *transparent_tiles.get(&pos).unwrap_or(&OPAQUE_ALPHA);

                    if tile_color.0.alpha() != target_alpha {
                        tile_color.0.set_alpha(target_alpha);
                    }
                }
            }
        }
    }
}
