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
    pub entity: Entity,
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct GetCloseToEntityAction;
