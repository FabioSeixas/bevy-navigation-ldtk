use bevy::prelude::*;

use crate::brain::cleanup::StartAction;

use super::components::ActiveRelaxTask;

fn cleanup_relaxing(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<ActiveRelaxTask>();
}

fn setup_relaxing(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(ActiveRelaxTask);
}

pub fn get_start_relax_action() -> StartAction {
    StartAction {
        cleanup: cleanup_relaxing,
        setup: setup_relaxing,
        title: "Relax"
    }
}
