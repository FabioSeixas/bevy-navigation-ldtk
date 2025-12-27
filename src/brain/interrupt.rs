use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    brain::{cleanup::CurrentTaskCleanup, events::InterruptCurrentTaskEvent},
    log::custom_debug,
};

#[derive(Clone, Component, Debug)]
pub struct InterruptCurrentTaskMarker;

pub fn interrupt_current_task_observer(
    event: On<InterruptCurrentTaskEvent>,
    mut commands: Commands,
) {
    commands
        .entity(event.entity)
        .insert(InterruptCurrentTaskMarker);
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct InterruptCurrentTaskScorer;

pub fn interrupt_current_task_scorer_system(
    agent_q: Query<(Entity, Option<&CurrentTaskCleanup>), With<InterruptCurrentTaskMarker>>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<InterruptCurrentTaskScorer>>,
    mut commands: Commands,
) {
    for (Actor(actor), mut score, _span) in &mut query {
        if let Ok((entity, cleanup)) = agent_q.get(*actor) {
            custom_debug(
                entity,
                "interrupt_current_task_scorer_system",
                "interrupt current action".into(),
            );

            if let Some(cleanup) = cleanup {
                (cleanup.0)(&mut commands, entity);
                commands.entity(entity).remove::<CurrentTaskCleanup>();
            }

            score.set(1.);

            commands
                .entity(entity)
                .remove::<InterruptCurrentTaskMarker>();
        } else {
            score.set(0.)
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct InterruptCurrentTaskAction;

pub fn interrupt_action_system(mut query: Query<(&mut ActionState, &InterruptCurrentTaskAction)>) {
    for (mut state, _) in &mut query {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Success;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
