use bevy::prelude::*;
use big_brain::prelude::*;
use rand::Rng;

use crate::{
    agent::Agent,
    interaction::{ActivelyInteracting, WaitingAsTarget},
    interaction_queue::AgentInteractionQueue,
    log::custom_debug,
};

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct HungryScorer;

pub fn hungry_scorer_system(
    agent_q: Query<&Agent>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<HungryScorer>>,
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
pub struct RelaxScorer;

pub fn relax_scorer_system(
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
pub struct TalkScorer;

pub fn talk_scorer_system(
    agent_q: Query<Entity, With<Agent>>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<TalkScorer>>,
) {
    for (Actor(actor), mut score, span) in &mut query {
        if let Ok(_) = agent_q.get(*actor) {
            score.set(get_probability(0.5));
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct ReiceiveInteractionScorer;

pub fn receive_interaction_scorer_system(
    agent_q: Query<
        &AgentInteractionQueue,
        (
            With<Agent>,
            Without<WaitingAsTarget>,
            Without<ActivelyInteracting>,
        ),
    >,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<ReiceiveInteractionScorer>>,
) {
    for (Actor(actor), mut score, span) in &mut query {
        // when not found (WaitingAsTarget is present) it will keep the last value used (I hope)
        if let Ok(agent_interaction_queue) = agent_q.get(*actor) {
            if agent_interaction_queue.is_empty() {
                custom_debug(
                    *actor,
                    "receive_interaction_scorer_system",
                    format!("queue is empty"),
                );
                score.set(0.);
            } else {
                custom_debug(
                    *actor,
                    "receive_interaction_scorer_system",
                    format!("queue is NOT empty"),
                );
                score.set(1.);
            }
        }
    }
}
