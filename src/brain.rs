use bevy::prelude::*;
use big_brain::prelude::*;

use crate::agent::{Agent, DefineRandomDestination, Walking};

pub struct BrainPlugin;

impl Plugin for BrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigBrainPlugin::new(PreUpdate)).add_systems(
            Update,
            (
                relax_action_system,
                relax_scorer_system,
                attach_main_thinker_to_agents,
            ),
        );
    }
}

fn attach_main_thinker_to_agents(
    mut commands: Commands,
    agent_q: Query<Entity, (With<Agent>, Without<Thinker>)>,
) {
    for entity in agent_q {
        commands.entity(entity).insert(
            Thinker::build()
                .label("Main Thinker")
                // Priority 1: Handle urgent needs with `When`. This will INTERRUPT other tasks.
                // .when(IsHungry, EatFoodAction::new())
                // Priority 2: Normal, scored behaviors. `pick` runs the action with the highest score.
                .picker(Highest)
                .when(RelaxScorer, RelaxAction),
        );
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
struct RelaxScorer;

fn relax_scorer_system(
    agent_q: Query<Entity, (With<Agent>, Without<Walking>)>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<RelaxScorer>>,
) {
    for (Actor(actor), mut score, span) in &mut query {
        if let Ok(_) = agent_q.get(*actor) {
            // The score here must be between 0.0 and 1.0.
            score.set(0.1);
            span.span().in_scope(|| {
                debug!("relax score set to 0.1");
            });
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
struct RelaxAction;

fn relax_action_system(
    // We execute actions by querying for their associated Action Component
    // (RelaxAction in this case). You'll always need both Actor and ActionState.
    agent_q: Query<Entity, (With<Agent>, Without<Walking>)>,
    mut query: Query<(&Actor, &mut ActionState, &RelaxAction, &ActionSpan)>,
    mut commands: Commands,
) {
    for (Actor(actor), mut state, _relax_action, span) in &mut query {
        // This sets up the tracing scope. Any `debug` calls here will be
        // spanned together in the output.
        let _guard = span.span().enter();

        match *state {
            ActionState::Requested => {
                debug!("Time to walk for relax!");
                commands.trigger(DefineRandomDestination { entity: *actor });
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                trace!("Walking...");
                if let Ok(_) = agent_q.get(*actor) {
                    debug!("Done walking");
                    *state = ActionState::Success;
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
