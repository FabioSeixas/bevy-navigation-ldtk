use std::time::Duration;

use bevy::prelude::*;
use big_brain::prelude::*;

use crate::world::components::GridPosition;

#[derive(Component)]
pub struct Walking {
    pub destination: GridPosition,
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct WalkingAction {
    pub destination: Option<GridPosition>,
}

impl WalkingAction {
    pub fn random_destination() -> Self {
        Self { destination: None }
    }

    pub fn destination(p: GridPosition) -> Self {
        Self {
            destination: Some(p),
        }
    }
}

#[derive(Component)]
pub struct GetCloseToEntity {
    pub target: Entity,
    pub recalculate_timer: Timer,
}

impl GetCloseToEntity {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            recalculate_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct GetCloseToEntityAction;
