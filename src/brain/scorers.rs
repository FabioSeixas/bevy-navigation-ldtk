use bevy::prelude::*;
use big_brain::prelude::*;
use rand::Rng;

use crate::{
    agent::Agent,
    brain::components::ActiveRelaxTask,
    interaction::{ActivelyInteracting, WaitingAsSource, WaitingAsTarget}, log::custom_debug,
};

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct HungryScorer;

pub fn hungry_scorer_system(
    agent_q: Query<&Agent>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<HungryScorer>>,
) {
    for (Actor(actor), mut score, _span) in &mut query {
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
    agent_q: Query<
        (Entity, Option<&ActiveRelaxTask>),
        (
            With<Agent>,
            Without<WaitingAsSource>,
            Without<WaitingAsTarget>,
            Without<ActivelyInteracting>,
        ),
    >,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<RelaxScorer>>,
) {
    for (Actor(actor), mut score, _span) in &mut query {
        if let Ok((_, relaxing)) = agent_q.get(*actor) {
            if relaxing.is_some() {
                score.set(1.0);
            } else {
                let prob = get_probability(0.5);

                // custom_debug(*actor, "relax_scorer_system", format!("prob: {}", prob));

                score.set(prob);
            }
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct TalkScorer;

pub fn talk_scorer_system(
    agent_q: Query<
        Entity,
        (
            With<Agent>,
            Without<WaitingAsSource>,
            Without<WaitingAsTarget>,
            Without<ActivelyInteracting>,
        ),
    >,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<TalkScorer>>,
) {
    for (Actor(actor), mut score, _span) in &mut query {
        if let Ok(_) = agent_q.get(*actor) {
            score.set(get_probability(0.5));
        } else {
            score.set(0.);
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct HandleAnyInteractionScorer;

pub fn handle_any_interaction_scorer_system(
    agent_q: Query<
        Entity,
        (
            With<Agent>,
            Or<(
                With<WaitingAsTarget>,
                With<WaitingAsSource>,
                With<ActivelyInteracting>,
            )>,
        ),
    >,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<HandleAnyInteractionScorer>>,
) {
    for (Actor(actor), mut score, _span) in &mut query {
        if let Ok(_) = agent_q.get(*actor) {
            score.set(1.0);
        } else {
            score.set(0.);
        }
    }
}
