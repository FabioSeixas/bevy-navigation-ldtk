use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    agent::Agent,
    interaction_queue::{AgentInteractionItem, AgentInteractionQueue},
    walk::components::GetCloseToEntity,
    world::components::GridPosition,
};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(start_interaction)
            .add_systems(
                PreUpdate,
                start_interaction_action_system.in_set(BigBrainSet::Actions),
            )
            .add_systems(Update, check_interaction_wait_timeout);
    }
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
                    info!(
                        "Interaction {} between {} and {} timed out (SourceWaitingForTarget)",
                        entity, interaction.source, interaction.target
                    );

                    commands.entity(entity).despawn();
                };
            }
            InteractionState::TargetWaitingForSource { timeout } => {
                if timeout.tick(time.delta()).just_finished() {
                    info!(
                        "Interaction {} between {} and {} timed out (TargetWaitingForSource)",
                        entity, interaction.source, interaction.target
                    );

                    commands.entity(entity).despawn();
                };
            }
            InteractionState::Active { duration } => {
                if duration.tick(time.delta()).just_finished() {
                    commands.entity(entity).despawn();
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

        commands.entity(trigger.source).insert(GetCloseToEntity {
            entity: trigger.target,
        });

        // send interaction request to target
        agent_interation_queue.add(AgentInteractionItem {
            interaction_entity,
            target: trigger.target,
            source: trigger.source,
        });
    }
}

fn start_interaction_action_system(
    interaction_q: Query<&Interaction>,
    source_agent_q: Query<&GridPosition, With<Agent>>,
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
                // Confirm that Agent is not already source in some interaction
                for interaction in interaction_q {
                    if interaction.source.eq(&source_entity) {
                        info!("Agent is already source in another interaction");
                        *state = ActionState::Failure;
                        return;
                    }
                }

                if let Ok(source_position) = source_agent_q.get(source_entity) {
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
                        *state = ActionState::Success;
                    } else {
                        *state = ActionState::Failure;
                    }
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Cancelled => {
                info!("start interaction was cancelled");
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
