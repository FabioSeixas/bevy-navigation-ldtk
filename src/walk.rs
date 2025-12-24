use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    agent::Agent,
    world::{
        components::{GridPosition, Occupied, Tile},
        grid::Grid,
        spatial_idx::SpatialIndex,
    },
};

pub struct WalkPlugin;

impl Plugin for WalkPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(define_random_destination)
            .add_systems(Update, walking_action_system);
    }
}

#[derive(Component)]
pub struct Walking {
    pub destination: GridPosition,
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct WalkingAction {
    destination: Option<GridPosition>,
}

impl WalkingAction {
    pub fn random_destination() -> Self {
        Self { destination: None }
    }

    pub fn destination(p: GridPosition) -> Self {
        Self {
            destination: Some(p),
        }
    }
}

fn walking_action_system(
    agent_q: Query<(&Walking, &GridPosition), With<Agent>>,
    mut query: Query<(&Actor, &mut ActionState, &WalkingAction, &ActionSpan)>,
    mut commands: Commands,
) {
    for (Actor(actor), mut state, walk_action, span) in &mut query {
        let _guard = span.span().enter();

        let entity = *actor;

        match *state {
            ActionState::Requested => {
                debug!("Time to walk for relax!");

                if let Some(destination) = &walk_action.destination {
                    commands.entity(entity).insert(Walking {
                        destination: destination.clone(),
                    });
                } else {
                    commands.trigger(DefineRandomDestination { entity });
                };
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                trace!("Walking...");

                if let Ok((walking, grid_position)) = agent_q.get(entity) {
                    if grid_position.eq(&walking.destination) {
                        debug!("Done walking");
                        commands.entity(entity).remove::<Walking>();
                        *state = ActionState::Success;
                    }
                }
            }
            // All Actions should make sure to handle cancellations!
            ActionState::Cancelled => {
                debug!("Action was cancelled. Considering this a failure.");
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

#[derive(Event, Debug)]
pub struct DefineRandomDestination {
    pub entity: Entity,
}

fn define_random_destination(
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
