use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    agent::Agent,
    interaction_queue::{AgentInteractionItem, AgentInteractionQueue},
    log::custom_debug,
    walk::components::GetCloseToEntity,
    world::components::GridPosition,
};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(start_interaction)
            .add_systems(
                PreUpdate,
                (
                    start_interaction_action_system,
                    receive_interaction_action_system,
                )
                    .in_set(BigBrainSet::Actions),
            )
            .add_systems(
                Update,
                (
                    check_interaction_wait_timeout,
                    activate_pending_interactions,
                ),
            );
    }
}

#[derive(Event)]
pub struct InteractionTimedOut {
    pub entity: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct ActivelyInteracting(Entity);

#[derive(Component, Clone, Copy)]
pub struct WaitingAsTarget(Entity);

#[derive(Component, Clone, Copy)]
pub struct WaitingAsSource(Entity);

fn check_interaction_wait_timeout(
    mut query: Query<(Entity, &Interaction, &mut InteractionState)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, interaction, mut interaction_state) in &mut query {
        match interaction_state.as_mut() {
            InteractionState::SourceWaitingForTarget { timeout } => {
                if timeout.tick(time.delta()).just_finished() {

                    custom_debug(
                        entity,
                        "check_interaction_wait_timeout",
                        format!(
                            "Interaction {} between {} and {} timed out (SourceWaitingForTarget)",
                            entity, interaction.source, interaction.target
                        ),
                    );

                    commands.entity(entity).despawn();

                    commands.trigger(InteractionTimedOut {
                        entity: interaction.source,
                    });
                };
            }
            InteractionState::TargetWaitingForSource { timeout } => {
                if timeout.tick(time.delta()).just_finished() {
                    info!(
                        "Interaction {} between {} and {} timed out (TargetWaitingForSource)",
                        entity, interaction.source, interaction.target
                    );

                    commands.entity(entity).despawn();

                    commands.trigger(InteractionTimedOut {
                        entity: interaction.source,
                    });
                };
            }
            InteractionState::Active { duration } => {
                if duration.tick(time.delta()).just_finished() {
                    info!(
                        "Interaction {} between {} and {} timed out (Active)",
                        entity, interaction.source, interaction.target
                    );
                    commands.entity(entity).despawn();

                    commands.trigger(InteractionTimedOut {
                        entity: interaction.source,
                    });
                };
            }
            InteractionState::Running { duration } => {
                if duration.tick(time.delta()).just_finished() {
                    commands.entity(entity).despawn();
                } else {
                    // println!("Running interaction: {:?}", duration.elapsed());
                };
            }
            InteractionState::Finished(result) => {
                info!("Finish interaction: {:?} with result {:?}", entity, result);
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct ReceiveInteractionAction;

fn receive_interaction_action_system(
    mut interaction_q: Query<(Entity, &mut InteractionState, &Interaction)>,
    mut agent_q: Query<
        (
            &mut AgentInteractionQueue,
            Option<&WaitingAsTarget>,
            Option<&ActivelyInteracting>,
        ),
        With<Agent>,
    >,
    mut query: Query<(
        &Actor,
        &mut ActionState,
        &mut ReceiveInteractionAction,
        &ActionSpan,
    )>,
    mut commands: Commands,
) {
    for (Actor(actor), mut state, _, span) in &mut query {
        let _guard = span.span().enter();

        let entity = *actor;

        match *state {
            ActionState::Requested => {
                if let Ok((mut agent_interation_queue, _, _)) = agent_q.get_mut(entity) {
                    if let Some(interaction_item) = agent_interation_queue.pop_first() {
                        if let Ok((interaction_entity, mut interaction_state, _interaction)) =
                            interaction_q.get_mut(interaction_item.interaction_entity)
                        {
                            info!(
                                "Interaction {} between {} and {} is received by target",
                                interaction_entity,
                                interaction_item.source,
                                interaction_item.target
                            );

                            // let (_, target_label) = interaction.get_kind_labels();

                            commands
                                .entity(interaction_item.target)
                                .insert(WaitingAsTarget(interaction_entity));

                            // .trigger(UpdateText {
                            //     content: format!("{} WaitingAsTarget", target_label),
                            // });

                            // if let Some(mut action) = maybe_action {
                            //     action.pause();
                            // }

                            *interaction_state = InteractionState::TargetWaitingForSource {
                                timeout: Timer::from_seconds(10., TimerMode::Once),
                            };

                            *state = ActionState::Executing;

                            // only one agent will be able to receive an interaction by frame
                            return;
                        } else {
                            *state = ActionState::Failure;
                        }
                    };
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if let Ok((_, maybe_waiting_as_target, maybe_actively_interacting)) =
                    agent_q.get(entity)
                {
                    if let Some(waiting_as_source) = maybe_waiting_as_target {
                        if let Ok(_) = interaction_q.get(waiting_as_source.0) {
                            // interaction running
                        } else {
                            info!("Interaction not found while WaitingAsTarget");
                            *state = ActionState::Failure;
                        }
                    } else if let Some(actively_interacting) = maybe_actively_interacting {
                        if let Ok(_) = interaction_q.get(actively_interacting.0) {
                            // interaction running
                        } else {
                            info!("Interaction not found while ActivelyInteracting");
                            *state = ActionState::Failure;
                        }
                    } else {
                        info!("target is neither WaitingAsSource and ActivelyInteracting");
                        *state = ActionState::Failure;
                    }
                }
            }
            ActionState::Cancelled => {
                info!("receive interaction was cancelled");
                *state = ActionState::Failure;
            }
            ActionState::Failure => {
                info!("receive interaction failure");
                commands
                    .entity(entity)
                    .remove::<(WaitingAsTarget, ActivelyInteracting)>();
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct StartInteractionAction;

fn find_agent_near(
    source_entity: Entity,
    source_agent_position: GridPosition,
    target_agent_query: Query<(Entity, &GridPosition), With<Agent>>,
) -> Option<(Entity, GridPosition, f32)> {
    let mut best: Option<(Entity, GridPosition, f32)> = None;

    for (entity, target_grid_position) in &target_agent_query {
        if entity.eq(&source_entity) {
            continue;
        }

        let d2 = source_agent_position.calc_distance(&target_grid_position);

        // TODO: set a maximum acceptable distance
        match best {
            None => best = Some((entity, target_grid_position.clone(), d2)),
            Some((_, _, best_d2)) => {
                if d2 < best_d2 {
                    best = Some((entity, target_grid_position.clone(), d2))
                }
            }
        }
    }

    best
}

#[derive(Event)]
pub struct StartInteraction {
    // pub kind: InteractionKind,
    pub source: Entity,
    pub target: Entity,
}

#[derive(Component)]
pub enum InteractionState {
    SourceWaitingForTarget { timeout: Timer },
    TargetWaitingForSource { timeout: Timer },
    Active { duration: Timer },
    Running { duration: Timer },
    Finished(FinishInteractionResult),
}

#[derive(Clone, Debug)]
pub enum FinishInteractionResult {
    Success,
    Failure,
}

#[derive(Component, Debug)]
pub struct Interaction {
    pub source: Entity,
    pub target: Entity,
    // pub kind: InteractionKind,
}

fn start_interaction(
    trigger: On<StartInteraction>,
    mut commands: Commands,
    mut target_agent_query: Query<&mut AgentInteractionQueue>,
) {
    if let Ok(mut agent_interation_queue) = target_agent_query.get_mut(trigger.target) {
        let interaction_entity = commands
            .spawn((
                Interaction {
                    source: trigger.source,
                    target: trigger.target,
                    // kind: trigger.kind.clone(),
                },
                InteractionState::SourceWaitingForTarget {
                    timeout: Timer::from_seconds(10., TimerMode::Once),
                },
            ))
            .id();

        info!(
            "interaction {} between {} and {} started",
            interaction_entity, trigger.source, trigger.target
        );

        // let text = match trigger.kind {
        //     InteractionKind::Talk => "Talking",
        //     InteractionKind::Trade => "Buying",
        //     InteractionKind::Question => "Asking",
        //     InteractionKind::Order => "Ordering",
        // };

        commands.entity(trigger.source).insert((
            WaitingAsSource(interaction_entity),
            GetCloseToEntity::new(trigger.target),
        ));

        // send interaction request to target
        agent_interation_queue.add(AgentInteractionItem {
            interaction_entity,
            target: trigger.target,
            source: trigger.source,
        });
    }
}

fn start_interaction_action_system(
    interaction_q: Query<&InteractionState>,
    source_agent_q: Query<
        (
            &GridPosition,
            Option<&WaitingAsSource>,
            Option<&ActivelyInteracting>,
        ),
        With<Agent>,
    >,
    target_agent_q: Query<(Entity, &GridPosition), With<Agent>>,
    mut query: Query<(
        &Actor,
        &mut ActionState,
        &mut StartInteractionAction,
        &ActionSpan,
    )>,
    mut commands: Commands,
) {
    for (Actor(actor), mut state, _, span) in &mut query {
        let _guard = span.span().enter();

        let source_entity = *actor;

        match *state {
            ActionState::Requested => {
                if let Ok((source_position, maybe_waiting_as_source, maybe_actively_interacting)) =
                    source_agent_q.get(source_entity)
                {
                    // Confirm that Agent is not already source in some interaction
                    if maybe_waiting_as_source.is_some() {
                        *state = ActionState::Failure;
                        continue;
                    }

                    // Confirm that Agent is not already interacting
                    if maybe_actively_interacting.is_some() {
                        *state = ActionState::Failure;
                        continue;
                    }

                    // This code is starting interaction with a random agent
                    // Not generic enough to be used to start other types of interactions
                    if let Some((target_entity, _, _)) =
                        find_agent_near(source_entity, source_position.clone(), target_agent_q)
                    {
                        commands.trigger(StartInteraction {
                            // kind: InteractionKind::Trade,
                            source: source_entity,
                            target: target_entity,
                        });
                        *state = ActionState::Executing;
                    } else {
                        *state = ActionState::Failure;
                    }
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if let Ok((_, maybe_waiting_as_source, maybe_actively_interacting)) =
                    source_agent_q.get(source_entity)
                {
                    if let Some(waiting_as_source) = maybe_waiting_as_source {
                        if let Ok(_) = interaction_q.get(waiting_as_source.0) {
                            // interaction running
                        } else {
                            info!("Interaction not found while WaitingAsSource");
                            *state = ActionState::Failure;
                        }
                    } else if let Some(actively_interacting) = maybe_actively_interacting {
                        if let Ok(_) = interaction_q.get(actively_interacting.0) {
                            // interaction running
                        } else {
                            info!("Interaction not found while ActivelyInteracting");
                            *state = ActionState::Failure;
                        }
                    } else {
                        info!("source is neither WaitingAsSource and ActivelyInteracting");
                        *state = ActionState::Failure;
                    }
                }
            }
            ActionState::Cancelled => {
                info!("start interaction was cancelled");
                *state = ActionState::Failure;
            }
            ActionState::Failure => {
                info!("start interaction failure");
                commands
                    .entity(source_entity)
                    .remove::<(WaitingAsSource, ActivelyInteracting)>();
            }
            _ => {}
        }
    }
}

fn activate_pending_interactions(
    mut query: Query<(Entity, &Interaction, &mut InteractionState)>,
    agent_query: Query<&GridPosition, Without<ActivelyInteracting>>,
    mut commands: Commands,
) {
    for (interaction_entity, interaction, mut interaction_state) in &mut query {
        match interaction_state.as_mut() {
            InteractionState::TargetWaitingForSource { timeout: _ } => {
                let mut should_start_interaction = false;

                if let Ok(source_position) = agent_query.get(interaction.source) {
                    if let Ok(target_position) = agent_query.get(interaction.target) {
                        if source_position.is_adjacent(target_position) {
                            should_start_interaction = true;
                        } else {
                            info!(
                                "TargetWaitingForSource: source not close enough. Target: {}. Source: {}.",
                                interaction.target, interaction.source
                            );
                        }
                    }
                }

                if should_start_interaction {
                    info!(
                        "Interaction {} between {} and {} is activating",
                        interaction_entity, interaction.source, interaction.target
                    );

                    let active_interaction = ActivelyInteracting(interaction_entity);
                    // let (source_label, target_label) = interaction.get_kind_labels();

                    commands
                        .entity(interaction.source)
                        .remove::<WaitingAsSource>()
                        .insert(active_interaction.clone());

                    commands
                        .entity(interaction.target)
                        .remove::<WaitingAsTarget>()
                        .insert(active_interaction);

                    *interaction_state = InteractionState::Active {
                        duration: Timer::from_seconds(5., TimerMode::Once),
                    };
                }
            }
            _ => {}
        }
    }
}
