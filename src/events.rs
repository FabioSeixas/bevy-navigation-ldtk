use bevy::prelude::*;

#[derive(Event, Message, Clone)]
pub struct AgentEnteredTile {
    pub entity: Entity,
}

#[derive(Event, Message, Clone)]
pub struct AgentLeftTile {
    pub entity: Entity,
}

#[derive(Event, Message, Clone)]
pub struct FaceToFaceEvent {
    pub agent_a: Entity,
    pub agent_b: Entity,
}
