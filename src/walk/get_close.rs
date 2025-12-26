use bevy::prelude::*;

use crate::{
    agent::Agent,
    interaction::InteractionFinished,
    log::custom_debug,
    walk::components::{GetCloseToEntity, Walking},
    world::{components::GridPosition, spatial_idx::SpatialIndex},
};

pub fn clean_get_close_to_entity_observer(
    event: On<InteractionFinished>,
    agent_q: Query<&GetCloseToEntity, With<Agent>>,
    mut commands: Commands,
) {
    let entity = event.source;
    if let Ok(_) = agent_q.get(entity) {
        custom_debug(
            entity,
            "clean_get_close_to_entity_observer",
            "GetCloseToEntity: Cleaning on InteractionFinished".into(),
        );

        commands
            .entity(entity)
            .remove::<(Walking, GetCloseToEntity)>();
    }
}

pub fn get_close_to_entity_system(
    agent_q: Query<(Entity, &GridPosition, Option<&Walking>, &GetCloseToEntity), With<Agent>>,
    target_agent_q: Query<&GridPosition, With<Agent>>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
) {
    for (entity, source_current_position, maybe_walking, get_close_to_entity) in agent_q {
        let mut clean_up = false;
        if let Some(walking) = maybe_walking {
            if source_current_position.eq(&walking.destination) {
                custom_debug(entity, "get_close_to_entity_system", "Done walking".into());

                // Check if target position still the same
                if let Ok(target_position) = target_agent_q.get(get_close_to_entity.target) {
                    if source_current_position.is_adjacent(target_position) {
                        // success, clean everything
                        clean_up = true;
                    } else {
                        custom_debug(
                            entity,
                            "get_close_to_entity_system",
                            "Target moved, walking again".into(),
                        );
                        commands.entity(entity).remove::<Walking>();
                    }
                } else {
                    custom_debug(
                        entity,
                        "get_close_to_entity_system",
                        "Target not found".into(),
                    );
                    clean_up = true;
                }
            } else {
                if get_close_to_entity.recalculate_timer.is_finished() {
                    custom_debug(entity, "get_close_to_entity_system", "Recalculating".into());
                    commands.entity(entity).remove::<Walking>();
                }
            }
        } else {
            if let Ok(target_position) = target_agent_q.get(get_close_to_entity.target) {
                if target_position.is_adjacent(source_current_position) {
                    clean_up = true;
                } else {
                    for destination_option in
                        source_current_position.get_ordered_neighbors(target_position)
                    {
                        if let Some(tile_data) =
                            spatial_idx.get_tile_data(destination_option.x, destination_option.y)
                        {
                            if tile_data.is_valid_destination() {
                                custom_debug(
                                    entity,
                                    "get_close_to_entity_system",
                                    "Start walking".into(),
                                );

                                commands.entity(entity).insert(Walking {
                                    destination: destination_option,
                                });

                                // break the for loop
                                break;
                            }
                        }
                    }
                }
            } else {
                custom_debug(
                    entity,
                    "get_close_to_entity_system",
                    "Target not found".into(),
                );
                clean_up = true;
            }
        }

        if clean_up {
            custom_debug(entity, "get_close_to_entity_system", "clean up".into());
            commands
                .entity(entity)
                .remove::<(Walking, GetCloseToEntity)>();
        }
    }
}

pub fn tick_get_close_to_entity_system(query: Query<&mut GetCloseToEntity>, time: Res<Time>) {
    for mut get_close_to_entity in query {
        get_close_to_entity.recalculate_timer.tick(time.delta());
    }
}
