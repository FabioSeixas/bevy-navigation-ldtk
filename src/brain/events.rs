use bevy::prelude::*;

#[derive(Event)]
pub struct InterruptCurrentTaskEvent {
    pub entity: Entity,
}
