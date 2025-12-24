use bevy::prelude::*;
use big_brain::prelude::*;

use crate::agent::Agent;

pub struct ConsumePlugin;

impl Plugin for ConsumePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, consuming_action_system.in_set(BigBrainSet::Actions));
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct ConsumeAction {
    duration: Timer,
}

impl ConsumeAction {
    pub fn new() -> Self {
        Self {
            duration: Timer::from_seconds(3., TimerMode::Once),
        }
    }
}

fn consuming_action_system(
    time: Res<Time>,
    mut agent_q: Query<&mut Agent>,
    mut query: Query<(&Actor, &mut ActionState, &mut ConsumeAction, &ActionSpan)>,
) {
    for (Actor(actor), mut state, mut consume_action, span) in &mut query {
        let _guard = span.span().enter();

        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                // TO DO: update hungry every frame,
                if consume_action.duration.tick(time.delta()).just_finished() {
                    if let Ok(mut agent) = agent_q.get_mut(*actor) {
                        agent.fill_hungry();
                        info!("Done consuming");
                        *state = ActionState::Success;
                    }
                }
            }
            // All Actions should make sure to handle cancellations!
            ActionState::Cancelled => {
                info!("Consume was cancelled");
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
