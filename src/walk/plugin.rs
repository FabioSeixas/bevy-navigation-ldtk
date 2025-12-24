use bevy::prelude::*;
use big_brain::prelude::*;

use crate::walk::{
    get_close::get_close_to_action_system,
    walk::{define_random_destination, walking_action_system},
};

pub struct WalkPlugin;

impl Plugin for WalkPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(define_random_destination).add_systems(
            PreUpdate,
            (walking_action_system, get_close_to_action_system).in_set(BigBrainSet::Actions),
        );
    }
}
