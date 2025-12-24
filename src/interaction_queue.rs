use std::collections::VecDeque;

use bevy::prelude::*;

pub struct AgentInteractionItem {
    pub interaction_entity: Entity,
    pub source: Entity,
    pub target: Entity,
}

#[derive(Component)]
pub struct AgentInteractionQueue {
    received_as_target_queue: VecDeque<AgentInteractionItem>,
}

impl AgentInteractionQueue {
    pub fn new() -> Self {
        Self {
            received_as_target_queue: VecDeque::new(),
        }
    }

    pub fn add(&mut self, item: AgentInteractionItem) {
        self.received_as_target_queue.push_back(item);
    }

    pub fn is_empty(&self) -> bool {
        self.received_as_target_queue.is_empty()
    }

    pub fn pop_first(&mut self) -> Option<AgentInteractionItem> {
        match self.received_as_target_queue.pop_front() {
            None => None,
            Some(v) => Some(v),
        }
    }
}
