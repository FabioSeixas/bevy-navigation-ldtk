use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct AgentLeftTile {
    pub entity: Entity,
}

#[derive(Event, Debug)]
pub struct AgentEnteredTile {
    pub entity: Entity,
}
