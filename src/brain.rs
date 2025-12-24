use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{agent::Agent, walk::WalkingAction};

pub struct BrainPlugin;

impl Plugin for BrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigBrainPlugin::new(PreUpdate))
            .add_systems(Update, (relax_scorer_system, attach_main_thinker_to_agents));
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
                .when(RelaxScorer, WalkingAction::random_destination()),
        );
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
struct RelaxScorer;

fn relax_scorer_system(
    agent_q: Query<Entity, With<Agent>>,
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
