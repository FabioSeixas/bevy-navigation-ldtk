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
            .add_observer(clean_agents_on_interaction_finished_observer)
            .add_systems(
                PreUpdate,
                (talk_action_system, handle_any_interaction_action_system)
                    .in_set(BigBrainSet::Actions),
            )
            .add_systems(
                Update,
                (
                    check_interaction_wait_timeout,
                    activate_pending_interactions,
                    process_interaction_queue_system,
                ),
            );
    }
}

#[derive(Event)]
pub struct InteractionFinished {
    pub interaction_entity: Entity,
    pub target: Entity,
    pub source: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct ActivelyInteracting(Entity);

#[derive(Component, Clone, Copy)]
pub struct WaitingAsTarget(Entity);

#[derive(Component, Clone, Copy)]
pub struct WaitingAsSource(Entity);

fn clean_agents_on_interaction_finished_observer(
    event: On<InteractionFinished>,
    source_agent_q: Query<(Option<&WaitingAsSource>, Option<&ActivelyInteracting>), With<Agent>>,
    target_agent_q: Query<(Option<&WaitingAsTarget>, Option<&ActivelyInteracting>), With<Agent>>,
    mut commands: Commands,
) {
    if let Ok((maybe_waiting_as_target, maybe_actively_interacting)) =
        target_agent_q.get(event.target)
    {
        if let Some(waiting_as_target) = maybe_waiting_as_target {
            if waiting_as_target.0 == event.interaction_entity {
                commands.entity(event.target).remove::<WaitingAsTarget>();
            }
        }

        if let Some(actively_interacting) = maybe_actively_interacting {
            if actively_interacting.0 == event.interaction_entity {
                commands
                    .entity(event.target)
                    .remove::<ActivelyInteracting>();
            }
        }
    }

    if let Ok((maybe_waiting_as_source, maybe_actively_interacting)) =
        source_agent_q.get(event.source)
    {
        if let Some(waiting_as_source) = maybe_waiting_as_source {
            if waiting_as_source.0 == event.interaction_entity {
                commands.entity(event.source).remove::<WaitingAsSource>();
            }
        }

        if let Some(actively_interacting) = maybe_actively_interacting {
            if actively_interacting.0 == event.interaction_entity {
                commands
                    .entity(event.source)
                    .remove::<ActivelyInteracting>();
            }
        }
    }

    commands.entity(event.interaction_entity).despawn();
}

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

                    commands.trigger(InteractionFinished {
                        source: interaction.source,
                        target: interaction.target,
                        interaction_entity: entity,
                    });
                };
            }
            InteractionState::TargetWaitingForSource { timeout } => {
                if timeout.tick(time.delta()).just_finished() {
                    custom_debug(
                        entity,
                        "check_interaction_wait_timeout",
                        format!(
                            "Interaction {} between {} and {} timed out (TargetWaitingForSource)",
                            entity, interaction.source, interaction.target
                        ),
                    );

                    commands.trigger(InteractionFinished {
                        source: interaction.source,
                        target: interaction.target,
                        interaction_entity: entity,
                    });
                };
            }
            InteractionState::Active { duration } => {
                if duration.tick(time.delta()).just_finished() {
                    custom_debug(
                        entity,
                        "check_interaction_wait_timeout",
                        format!(
                            "Interaction {} between {} and {} timed out (Active)",
                            entity, interaction.source, interaction.target
                        ),
                    );
                    commands.trigger(InteractionFinished {
                        source: interaction.source,
                        target: interaction.target,
                        interaction_entity: entity,
                    });
                };
            }
            InteractionState::Running { duration } => {
                if duration.tick(time.delta()).just_finished() {
                    commands.trigger(InteractionFinished {
                        source: interaction.source,
                        target: interaction.target,
                        interaction_entity: entity,
                    });
                } else {
                    // println!("Running interaction: {:?}", duration.elapsed());
                };
            }
            InteractionState::Finished(result) => {
                info!("Finish interaction: {:?} with result {:?}", entity, result);
                commands.trigger(InteractionFinished {
                    source: interaction.source,
                    target: interaction.target,
                    interaction_entity: entity,
                });
            }
        }
    }
}

/// The system that will process the interaction queue for all agents.
fn process_interaction_queue_system(
    mut commands: Commands,
    mut agent_q: Query<
        (Entity, &mut AgentInteractionQueue, Option<&WaitingAsSource>),
        (Without<WaitingAsTarget>, Without<ActivelyInteracting>),
    >,
    mut interaction_q: Query<(&mut InteractionState, &Interaction)>,
) {
    for (agent_entity, mut agent_interaction_queue, maybe_source) in agent_q.iter_mut() {
        if let Some(interaction_item) = agent_interaction_queue.pop_first() {
            if let Ok((mut interaction_state, _)) =
                interaction_q.get_mut(interaction_item.interaction_entity)
            {
                custom_debug(
                    agent_entity,
                    "process_interaction_queue_system",
                    format!(
                        "Agent is accepting interaction {}",
                        interaction_item.interaction_entity
                    ),
                );

                // Add the WaitingAsTarget component to the agent.
                commands
                    .entity(agent_entity)
                    .insert(WaitingAsTarget(interaction_item.interaction_entity));

                *interaction_state = InteractionState::TargetWaitingForSource {
                    timeout: Timer::from_seconds(10., TimerMode::Once),
                };

                // If the agent is NOT already initiating an interaction, interrupt its current task.
                // if maybe_source.is_none() {
                //     custom_debug(
                //         agent_entity,
                //         "process_interaction_queue_system",
                //         "Agent is not a source, firing interrupt.".into(),
                //     );
                //     commands.trigger(InterruptCurrentTaskEvent {
                //         entity: agent_entity,
                //     });
                // } else {
                //     custom_debug(
                //         agent_entity,
                //         "process_interaction_queue_system",
                //         "Agent is already a source, not interrupting.".into(),
                //     );
                // }
            }
        }
    }
}

/// An action that just finds a target and triggers the StartInteraction event.
#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct TalkAction;

fn talk_action_system(
    agent_q: Query<(&GridPosition, Option<&WaitingAsSource>)>,
    target_q: Query<(Entity, &GridPosition), With<Agent>>,
    mut query: Query<(&Actor, &mut ActionState, &ActionSpan, &TalkAction)>,
    mut commands: Commands,
) {
    for (Actor(actor), mut state, _span, _) in &mut query {
        match *state {
            ActionState::Requested => {
                custom_debug(*actor, "talk_action_system", "action was requested".into());
                if let Ok((source_position, maybe_waiting_as_source)) = agent_q.get(*actor) {
                    if maybe_waiting_as_source.is_some() {
                        *state = ActionState::Failure;
                        continue;
                    }

                    if let Some((target_entity, _, _)) =
                        find_agent_near(*actor, source_position.clone(), target_q)
                    {
                        commands.trigger(StartInteraction {
                            source: *actor,
                            target: target_entity,
                        });
                        *state = ActionState::Success;
                    } else {
                        *state = ActionState::Failure;
                    }
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Cancelled => {
                custom_debug(*actor, "talk_action_system", "action was cancelled".into());
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

/// The single, all-encompassing action for being in any interaction state.
#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct HandleAnyInteractionAction;

fn handle_any_interaction_action_system(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState, &HandleAnyInteractionAction)>,
    agent_q: Query<(
        Option<&WaitingAsSource>,
        Option<&WaitingAsTarget>,
        Option<&ActivelyInteracting>,
    )>,
) {
    for (Actor(actor), mut state, _) in &mut query {
        match *state {
            ActionState::Requested => {
                custom_debug(
                    *actor,
                    "handle_any_interaction_action_system",
                    "Agent start handling interaction".into(),
                );
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                if let Ok((maybe_source, maybe_target, maybe_active)) = agent_q.get(*actor) {
                    // If all interaction components are gone, the action is over.
                    if maybe_source.is_none() && maybe_target.is_none() && maybe_active.is_none() {
                        custom_debug(
                            *actor,
                            "handle_any_interaction_action_system",
                            "Agent is neither WaitingAsSource, WaitingAsTarget and ActivelyInteracting".into(),
                        );
                        *state = ActionState::Success;
                    }
                } else {
                    // If the agent entity is gone for some reason, fail.
                    *state = ActionState::Failure;
                }
            }
            ActionState::Cancelled => {
                custom_debug(
                    *actor,
                    "handle_any_interaction_action_system",
                    "action was cancelled".into(),
                );
                // If the action is cancelled, we need to clean up all interaction components.
                commands.entity(*actor).remove::<(
                    WaitingAsSource,
                    WaitingAsTarget,
                    ActivelyInteracting,
                    GetCloseToEntity,
                )>();

                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

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

        custom_debug(
            interaction_entity,
            "start_interaction",
            format!(
                "interaction {} between {} and {} started",
                interaction_entity, trigger.source, trigger.target
            ),
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
                            // info!(
                            //     "TargetWaitingForSource: source not close enough. Target: {}. Source: {}.",
                            //     interaction.target, interaction.source
                            // );
                        }
                    }
                }

                if should_start_interaction {
                    custom_debug(
                        interaction_entity,
                        "activate_pending_interactions",
                        format!(
                            "Interaction {} between {} and {} is activating",
                            interaction_entity, interaction.source, interaction.target
                        ),
                    );

                    let active_interaction = ActivelyInteracting(interaction_entity);
                    // let (source_label, target_label) = interaction.get_kind_labels();

                    commands
                        .entity(interaction.source)
                        .remove::<(WaitingAsSource, GetCloseToEntity)>()
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
