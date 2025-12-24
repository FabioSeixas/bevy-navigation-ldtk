use bevy::prelude::*;
use big_brain::prelude::*;
use rand::Rng;

use crate::{
    agent::Agent,
    consume::ConsumeAction,
    interaction::StartInteractionAction,
    walk::components::{GetCloseToEntityAction, WalkingAction},
    world::grid::Grid,
};

pub struct BrainPlugin;

impl Plugin for BrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigBrainPlugin::new(PreUpdate))
            .add_systems(
                PreUpdate,
                (
                    hungry_scorer_system,
                    relax_scorer_system,
                    talk_scorer_system,
                )
                    .in_set(BigBrainSet::Scorers),
            )
            .add_systems(Update, attach_main_thinker_to_agents);
    }
}

fn attach_main_thinker_to_agents(
    mut commands: Commands,
    agent_q: Query<Entity, (With<Agent>, Without<Thinker>)>,
) {
    let hungry_location = Grid::get_random_position();
    for entity in agent_q {
        commands.entity(entity).insert(
            Thinker::build()
                .label("Main Thinker")
                // Priority 1: Handle urgent needs with `When`. This will INTERRUPT other tasks.
                .when(
                    Hungry,
                    Steps::build()
                        .label("WalkAndConsume")
                        .step(WalkingAction::destination(hungry_location.clone()))
                        .step(ConsumeAction::new()),
                )
                // Priority 2: Normal, scored behaviors. `pick` runs the action with the highest score.
                .picker(Highest)
                // .when(RelaxScorer, WalkingAction::random_destination())
                .when(
                    TalkScorer,
                    Steps::build()
                        .label("Talk")
                        .step(StartInteractionAction)
                        .step(GetCloseToEntityAction),
                ),
        );
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct Hungry;

pub fn hungry_scorer_system(
    agent_q: Query<&Agent>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<Hungry>>,
) {
    for (Actor(actor), mut score, span) in &mut query {
        if let Ok(agent) = agent_q.get(*actor) {
            if agent.is_hungry() {
                score.set(1.);
            } else {
                score.set(0.);
            }
        }
    }
}

fn get_probability(max: f32) -> f32 {
    let mut rnd = rand::thread_rng();
    rnd.gen_range((0.)..max)
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
struct RelaxScorer;

fn relax_scorer_system(
    agent_q: Query<Entity, With<Agent>>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<RelaxScorer>>,
) {
    for (Actor(actor), mut score, span) in &mut query {
        if let Ok(_) = agent_q.get(*actor) {
            score.set(get_probability(0.5));
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
struct TalkScorer;

fn talk_scorer_system(
    agent_q: Query<Entity, With<Agent>>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<TalkScorer>>,
) {
    for (Actor(actor), mut score, span) in &mut query {
        if let Ok(_) = agent_q.get(*actor) {
            score.set(get_probability(0.5));
        }
    }
}
