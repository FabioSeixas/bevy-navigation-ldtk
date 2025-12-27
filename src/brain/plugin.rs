use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    agent::Agent,
    brain::{cleanup::*, interrupt::*, relax::*, scorers::*},
    consume::ConsumeAction,
    interaction::{HandleAnyInteractionAction, TalkAction},
    walk::components::WalkingAction,
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
                    handle_any_interaction_scorer_system,
                )
                    .in_set(BigBrainSet::Scorers),
            )
            .add_systems(
                Update,
                (
                    start_action_system,
                    finish_action_system,
                    interrupt_action_system,
                )
                    .in_set(BigBrainSet::Actions),
            )
            .add_systems(Last, attach_main_thinker_to_agents);
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
                // P1: Highest priority interrupt.
                .when(InterruptCurrentTaskScorer, InterruptCurrentTaskAction)
                // P2: Handle any ongoing interaction state.
                .when(
                    HandleAnyInteractionScorer,
                    Steps::build()
                        .label("HandleAnyInteraction")
                        .step(StartAction::empty("HandleAnyInteraction"))
                        .step(HandleAnyInteractionAction),
                )
                // P3: Lowest priority general behaviors.
                .picker(Highest)
                .when(
                    HungryScorer,
                    Steps::build()
                        .label("Hungry")
                        .step(StartAction::empty("Hungry"))
                        .step(WalkingAction::destination(hungry_location.clone()))
                        .step(ConsumeAction::new()),
                )
                .when(
                    RelaxScorer,
                    Steps::build()
                        .label("Relax")
                        .step(get_start_relax_action())
                        .step(WalkingAction::random_destination())
                        .step(FinishAction),
                )
                .when(
                    TalkScorer,
                    Steps::build()
                        .label("Talk")
                        .step(StartAction::empty("Talk"))
                        .step(TalkAction),
                ),
        );
    }
}
