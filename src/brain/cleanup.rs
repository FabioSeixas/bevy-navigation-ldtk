use bevy::prelude::*;
use big_brain::prelude::*;

use crate::log::custom_debug;

#[derive(Component)]
pub struct CurrentTaskCleanup(pub fn(&mut Commands, Entity));

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct StartAction {
    pub setup: fn(&mut Commands, Entity),
    pub cleanup: fn(&mut Commands, Entity),
    pub title: &'static str,
}

impl StartAction {
    // Use this when you just want to run cleanup action at the beginning
    pub fn empty(title: &'static str) -> Self {
        Self {
            setup: |_: &mut Commands, _: Entity| {},
            cleanup: |_: &mut Commands, _: Entity| {},
            title,
        }
    }
}

// pub struct CurrentTaskCleanup(pub fn(&mut Commands, Entity));
pub fn start_action_system(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState, &StartAction)>,
    cleanup_q: Query<&CurrentTaskCleanup>,
) {
    for (Actor(actor), mut state, action) in query.iter_mut() {
        match *state {
            ActionState::Init => {
                // If there's an existing cleanup task, execute it and remove it.
                if let Ok(old_cleanup) = cleanup_q.get(*actor) {
                    (old_cleanup.0)(&mut commands, *actor);
                    commands.entity(*actor).remove::<CurrentTaskCleanup>();
                }

                custom_debug(
                    *actor,
                    "start_action_system",
                    format!("starting {}", action.title),
                );

                (action.setup)(&mut commands, *actor);
                commands
                    .entity(*actor)
                    .insert(CurrentTaskCleanup(action.cleanup));
            }
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

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct FinishAction;

pub fn finish_action_system(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<FinishAction>>,
    cleanup_q: Query<&CurrentTaskCleanup>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        match *state {
            ActionState::Requested => {
                if let Ok(old_cleanup) = cleanup_q.get(*actor) {
                    (old_cleanup.0)(&mut commands, *actor);
                    commands.entity(*actor).remove::<CurrentTaskCleanup>();
                }

                *state = ActionState::Success;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
