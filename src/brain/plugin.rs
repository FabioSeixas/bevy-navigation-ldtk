use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    agent::Agent,
    brain::{interrupt::*, scorers::*},
    consume::ConsumeAction,
    interaction::{ReceiveInteractionAction, StartInteractionAction},
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
                    receive_interaction_scorer_system,
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
                    ReiceiveInteractionScorer,
                    Steps::build()
                        .label("ReceiveInteraction")
                        .step(ReceiveInteractionAction)
                )
                // Priority 3:
                .when(
                    HungryScorer,
                    Steps::build()
                        .label("Hungry")
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
