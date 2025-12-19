use std::collections::{HashSet, VecDeque};

use bevy::prelude::*;

use crate::{
    constants::{GRID_HEIGHT, GRID_WIDTH, PATHFINDER_MAX_DEPTH},
    world::{components::*, spatial_idx::*},
};

#[derive(Clone, Debug)]
struct PathNode {
    position: GridPosition,
    g: f32,
    h: f32,
    f: f32,
    parent: Option<Box<PathNode>>,
}

impl PathNode {
    pub fn get_parent_rec(&self) -> Vec<GridPosition> {
        if let Some(parent) = self.parent.as_ref() {
            let mut v = parent.get_parent_rec();
            v.push(self.position.clone());
            v
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pathfinder {
    pub goal: GridPosition,
    pub closed_list: Vec<PathNode>,
    pub open_list: VecDeque<PathNode>,
    pub status: PathfinderStatus,
}

#[derive(Debug, Clone)]
enum PathfinderStatus {
    Calculating(usize), // current depth
    Finished(Vec<GridPosition>),
}

impl Pathfinder {
    pub fn new(start: &GridPosition, goal: &GridPosition) -> Self {
        let h = Pathfinder::calculate_heuristic(&start, &goal);

        let start_node = PathNode {
            g: 0.,
            h,
            f: h,
            position: start.clone(),
            parent: None,
        };

        let mut open_list = VecDeque::new();
        open_list.push_back(start_node);

        Self {
            goal: goal.clone(),
            closed_list: vec![],
            open_list,
            status: PathfinderStatus::Calculating(0),
        }
    }
    fn calculate_heuristic(pos1: &GridPosition, pos2: &GridPosition) -> f32 {
        ((pos2.x - pos1.x).pow(2) as f32 + ((pos2.y - pos1.y).pow(2) as f32)).sqrt()
    }

    fn get_nearby(reference_position: &GridPosition) -> Vec<GridPosition> {
        let mut nearby = Vec::new();
        for x in -1..2 {
            for y in -1..2 {
                let new_x = reference_position.x + x;
                let new_y = reference_position.y + y;

                // avoid include origin in nearby result
                if new_x == reference_position.x && new_y == reference_position.y {
                    continue;
                }

                if new_x < 0 || new_y < 0 {
                    continue;
                }

                if new_x > (GRID_WIDTH - 1) || new_y > (GRID_HEIGHT - 1) {
                    continue;
                }

                nearby.push(GridPosition { x: new_x, y: new_y });
            }
        }
        nearby
    }

    pub fn get_path_if_finished(&mut self) -> Option<Vec<GridPosition>> {
        if let PathfinderStatus::Finished(_) = self.status {
            if let PathfinderStatus::Finished(path) =
                std::mem::replace(&mut self.status, PathfinderStatus::Finished(vec![]))
            {
                return Some(path);
            }
        }
        None
    }

    pub fn get_current_node_position(&self) -> Option<&GridPosition> {
        match self.open_list.front() {
            Some(n) => Some(&n.position),
            None => None,
        }
    }

    pub fn step(
        &mut self,
        spatial_index: &SpatialIndex,
        dynamic_occupied_tiles: &HashSet<GridPosition>,
    ) {
        if let PathfinderStatus::Finished(_) = self.status {
            return;
        }

        // 1. no more nodes → fail
        let current_node = match self.open_list.pop_front() {
            Some(n) => n,
            None => {
                self.status = PathfinderStatus::Finished(vec![]);
                return;
            }
        };

        // 2. goal found
        if current_node.position == self.goal {
            self.status = PathfinderStatus::Finished(current_node.get_parent_rec());
            return;
        }

        // Check for max depth
        if let PathfinderStatus::Calculating(curr_depth) = &mut self.status {
            if *curr_depth > PATHFINDER_MAX_DEPTH {
                self.status = PathfinderStatus::Finished(current_node.get_parent_rec());
                return;
            }

            *curr_depth += 1;
        }

        // 3. expand node
        let current_idx = self.closed_list.len();
        self.closed_list.push(current_node);
        let current = &self.closed_list[current_idx];

        // 4. neighbors
        for pos in Pathfinder::get_nearby(&current.position) {
            // Dynamic Check
            if dynamic_occupied_tiles.contains(&pos) {
                continue;
            }

            // Static Check
            let current_tile_data = spatial_index
                .map
                .get(&(current.position.x, current.position.y))
                .unwrap();
            let neighbor_tile_data = spatial_index.map.get(&(pos.x, pos.y)).unwrap();

            if !current_tile_data.is_traversable_to(neighbor_tile_data) {
                continue;
            }

            // ignore closed
            if self.closed_list.iter().any(|n| n.position == pos) {
                continue;
            }

            let tentative_g = current.g + Pathfinder::calculate_heuristic(&current.position, &pos);

            let h = Pathfinder::calculate_heuristic(&pos, &self.goal);

            // already exists in open?
            if let Some(nei) = self.open_list.iter_mut().find(|n| n.position == pos) {
                if tentative_g < nei.g {
                    nei.g = tentative_g;
                    nei.f = tentative_g + h;
                    nei.parent = Some(Box::new(current.clone()));
                }
            } else {
                self.open_list.push_back(PathNode {
                    g: tentative_g,
                    h,
                    f: tentative_g + h,
                    position: pos,
                    parent: Some(Box::new(current.clone())),
                });

                // keep sorted by f
                let mut v: Vec<_> = self.open_list.drain(..).collect();
                v.sort_by(|a, b| a.f.total_cmp(&b.f));
                self.open_list = v.into_iter().collect();
            }
        }
    }
}
