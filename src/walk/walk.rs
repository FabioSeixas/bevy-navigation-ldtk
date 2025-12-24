use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    agent::Agent,
    walk::{components::{Walking, WalkingAction}, events::DefineRandomDestination},
    world::{
        components::{GridPosition, Occupied, Tile},
        grid::Grid,
        spatial_idx::SpatialIndex,
    },
};

pub fn walking_action_system(
    agent_q: Query<(&Walking, &GridPosition), With<Agent>>,
    mut query: Query<(&Actor, &mut ActionState, &WalkingAction, &ActionSpan)>,
    mut commands: Commands,
) {
    for (Actor(actor), mut state, walk_action, span) in &mut query {
        let _guard = span.span().enter();

        let entity = *actor;

        match *state {
            ActionState::Requested => {
                if let Some(destination) = &walk_action.destination {
                    info!("Walking to defined destination");
                    commands.entity(entity).insert(Walking {
                        destination: destination.clone(),
                    });
                } else {
                    info!("Walking to random destination");
                    commands.trigger(DefineRandomDestination { entity });
                };
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                if let Ok((walking, grid_position)) = agent_q.get(entity) {
                    if grid_position.eq(&walking.destination) {
                        info!("Done walking");
                        commands.entity(entity).remove::<Walking>();
                        *state = ActionState::Success;
                    }
                }
            }
            // All Actions should make sure to handle cancellations!
            ActionState::Cancelled => {
                info!("Walking was cancelled");
                commands.entity(entity).remove::<Walking>();
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

pub fn define_random_destination(
    event: On<DefineRandomDestination>,
    tile_query: Query<&Tile, Without<Occupied>>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
) {
    let agent_entity = event.entity;
    let mut chosen_destination_pos: Option<GridPosition> = None;
    while chosen_destination_pos.is_none() {
        let pos = Grid::get_random_position();
        if let Some(tile_data) = spatial_idx.map.get(&(pos.x, pos.y)) {
            if tile_data.is_valid_destination() {
                if let Ok(_) = tile_query.get(tile_data.entity) {
                    chosen_destination_pos = Some(pos);
                }
            }
        }
    }
    if let Some(destination_pos) = chosen_destination_pos {
        commands.entity(agent_entity).insert(Walking {
            destination: destination_pos,
        });
    }
}
