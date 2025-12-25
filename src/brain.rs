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
            .add_observer(interrupt_current_task_observer)
            .add_systems(
                PreUpdate,
                (
                    hungry_scorer_system,
                    relax_scorer_system,
                    talk_scorer_system,
                    interrupt_current_task_scorer_system,
                )
                    .in_set(BigBrainSet::Scorers),
            )
            .add_systems(
                Update,
                (attach_main_thinker_to_agents, interrupt_action_system),
            );
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
                // Priority 1: Will force the cancelation of anything running right now
                .when(InterruptCurrentTaskScorer, InterruptCurrentTaskAction)
                // Priority 2:
                .when(
                    HungryScorer,
                    Steps::build()
                        .label("WalkAndConsume")
                        .step(WalkingAction::destination(hungry_location.clone()))
                        .step(ConsumeAction::new()),
                )
                // Priority 3: Normal, scored behaviors. `pick` runs the action with the highest score.
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

#[derive(Clone, Component, Debug)]
pub struct InterruptCurrentTaskMarker;

#[derive(Event)]
pub struct InterruptCurrentTaskEvent {
    pub entity: Entity,
}

fn interrupt_current_task_observer(event: On<InterruptCurrentTaskEvent>, mut commands: Commands) {
    commands
        .entity(event.entity)
        .insert(InterruptCurrentTaskMarker);
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
struct InterruptCurrentTaskScorer;

fn interrupt_current_task_scorer_system(
    agent_q: Query<Entity, With<InterruptCurrentTaskMarker>>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<InterruptCurrentTaskScorer>>,
    mut commands: Commands,
) {
    for (Actor(actor), mut score, span) in &mut query {
        if let Ok(entity) = agent_q.get(*actor) {
            info!("Interrupting current action");

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

fn interrupt_action_system(mut query: Query<(&mut ActionState, &InterruptCurrentTaskAction)>) {
    for (mut state, _) in &mut query {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Success;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
struct HungryScorer;

fn hungry_scorer_system(
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
