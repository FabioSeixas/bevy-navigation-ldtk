use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    agent::Agent,
    walk::components::{GetCloseToEntityAction, GetCloseToEntity, Walking},
    world::components::GridPosition,
};

pub fn get_close_to_action_system(
    agent_q: Query<(&GridPosition, Option<&Walking>, Option<&GetCloseToEntity>), With<Agent>>,
    target_agent_q: Query<&GridPosition, With<Agent>>,
    mut query: Query<(&Actor, &mut ActionState, &GetCloseToEntityAction, &ActionSpan)>,
    mut commands: Commands,
) {
    for (Actor(actor), mut state, _get_close_to_action, span) in &mut query {
        let _guard = span.span().enter();

        let source_entity = *actor;

        match *state {
            ActionState::Requested => {
                if let Ok((_, _, maybe_get_close_to_entity)) = agent_q.get(source_entity) {
                    if let Some(get_close_to_entity) = maybe_get_close_to_entity {
                        if let Ok(target_position) = target_agent_q.get(get_close_to_entity.entity)
                        {
                            // FIX: destination can not be the same point as target. It must be:
                            // 1) an adjacent point
                            // 2) a valid point (not wall, not door, not furniture)
                            commands.entity(source_entity).insert(Walking {
                                destination: target_position.clone(),
                            });
                            *state = ActionState::Executing;
                        } else {
                            info!("Target not found");
                            *state = ActionState::Failure;
                        }
                    } else {
                        info!("GetCloseToEntity is None, wait next tick");
                    }
                } else {
                    info!("Source not found");
                    *state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if let Ok((source_current_position, maybe_walking, maybe_get_close_to_entity)) =
                    agent_q.get(source_entity)
                {
                    if let Some(walking) = maybe_walking {
                        if source_current_position.eq(&walking.destination) {
                            info!("Done walking");

                            let target = maybe_get_close_to_entity
                                .expect("get close to entity must not be None");

                            // Check if target position still the same
                            if let Ok(target_position) = target_agent_q.get(target.entity) {
                                // TODO: fix distance check here
                                if source_current_position.is_adjacent(target_position) {
                                    *state = ActionState::Success;
                                } else {
                                    info!("Target moved, walking again");
                                    commands.entity(source_entity).remove::<Walking>();
                                    *state = ActionState::Requested;
                                }
                            } else {
                                info!("Target not found");
                                *state = ActionState::Failure;
                            }
                        }
                    } else {
                        info!("Walking is None");
                        *state = ActionState::Failure;
                    }
                }
            }
            ActionState::Cancelled => {
                info!("GetCloseToAction was cancelled");
                *state = ActionState::Failure;
            }
            ActionState::Failure => {
                commands
                    .entity(source_entity)
                    .remove::<(Walking, GetCloseToEntity)>();
            }
            ActionState::Success => {
                commands
                    .entity(source_entity)
                    .remove::<(Walking, GetCloseToEntity)>();
            }
            _ => {}
        }
    }
}
