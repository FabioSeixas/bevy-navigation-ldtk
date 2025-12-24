use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct DefineRandomDestination {
    pub entity: Entity,
}
