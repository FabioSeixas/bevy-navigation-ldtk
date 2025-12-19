use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    constants::*,
    pathfinder::Pathfinder,
    world::{components::*, grid::*, spatial_idx::*},
};

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnAgentTimer(Timer::from_seconds(2.0, TimerMode::Once)))
            .add_observer(update_pathfinding_curr_step)
            .add_observer(pathfinding_finish_path_step)
            .add_observer(update_agent_position)
            .add_observer(on_update_agent_color)
            .add_systems(
                Update,
                (
                    define_destination_system,
                    check_reach_destination_system,
                    movement_agent,
                    check_agent_pathfinding,
                    spawn_agent_system,
                ),
            );
    }
}

#[derive(Resource)]
struct SpawnAgentTimer(Timer);

/// Marker for the agent
#[derive(Component)]
pub struct Agent {
    pathfinding_entity: Entity,
}

#[derive(Component)]
pub struct Walking {
    pub destination: GridPosition,
}

#[derive(Component, Default)]
pub enum AgentPathfinding {
    #[default]
    Nothing,
    Calculating(Pathfinder),
    Ready(AgentCurrentPath),
}

impl AgentPathfinding {
    pub fn start_path_calculation(
        &mut self,
        agent_curr_position: &GridPosition,
        destination: &GridPosition,
    ) {
        *self = AgentPathfinding::Calculating(Pathfinder::new(agent_curr_position, destination));
    }

    pub fn start_walking_path(&mut self, path: Vec<GridPosition>) {
        *self = AgentPathfinding::Ready(AgentCurrentPath {
            path,
            status: AgentCurrentPathStatus::WaitingNextStep((0, 0)),
        });
    }

    pub fn reset(&mut self) {
        *self = AgentPathfinding::Nothing;
    }
}

#[derive(Debug)]
pub struct AgentCurrentPath {
    path: Vec<GridPosition>,
    status: AgentCurrentPathStatus,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AgentCurrentPathStatus {
    WaitingNextStep((usize, usize)), // (step_idx, retry_count)
    RunningStep(usize),
}

fn spawn_agent_system(
    mut commands: Commands,
    time: Res<Time>,
    timer: Option<ResMut<SpawnAgentTimer>>,
    query: Query<&GridPosition, (With<Tile>, Without<Occupied>)>,
    spatial_idx: Res<SpatialIndex>,
) {
    if let Some(mut timer) = timer {
        if timer.0.tick(time.delta()).just_finished() {
            for _ in 0..AGENTS_COUNT {
                let mut done = false;
                while !done {
                    let grid_pos = Grid::get_random_position();
                    if let Some(tile_data) = spatial_idx.map.get(&(grid_pos.x, grid_pos.y)) {
                        if tile_data.tile_type == TileType::Outside {
                            if let Ok(_) = query.get(tile_data.entity) {
                                done = true;
                                let pos = Grid::grid_to_world(grid_pos.x, grid_pos.y);
                                let pathfinding_entity = commands
                                    .spawn((
                                        AgentPathfinding::default(),
                                        grid_pos.clone(),
                                        Sprite {
                                            color: Color::linear_rgb(1.0, 1.2, 1.2),
                                            custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)),
                                            ..default()
                                        },
                                        Transform::from_translation(Vec3 {
                                            x: pos.x,
                                            y: pos.y,
                                            z: PATHFINDER_Z_VALUE,
                                        }),
                                    ))
                                    .id();
                                commands.spawn((
                                    Agent { pathfinding_entity },
                                    grid_pos,
                                    Sprite {
                                        color: Color::linear_rgb(1.0, 0.2, 0.2),
                                        custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)),
                                        ..default()
                                    },
                                    Transform::from_translation(Vec3 {
                                        x: pos.x,
                                        y: pos.y,
                                        z: AGENT_Z_VALUE,
                                    }),
                                ));
                            }
                        }
                    }
                }
            }
            commands.remove_resource::<SpawnAgentTimer>();
        }
    }
}

fn define_destination_system(
    mut query: Query<Entity, (Without<Walking>, With<Agent>)>,
    tile_query: Query<&Tile, Without<Occupied>>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
) {
    for agent_entity in &mut query {
        let mut chosen_destination_pos: Option<GridPosition> = None;
        while chosen_destination_pos.is_none() {
            let pos = Grid::get_random_position();
            if let Some(tile_data) = spatial_idx.map.get(&(pos.x, pos.y)) {
                if tile_data.tile_type != TileType::Wall && tile_data.tile_type != TileType::Door {
                    if let Ok(_) = tile_query.get(tile_data.entity) {
                        chosen_destination_pos = Some(pos);
                    }
                }
            }
        }
        if let Some(destination_pos) = chosen_destination_pos {
            commands.entity(agent_entity).insert(Walking {
                destination: destination_pos,
            });
        }
    }
}

fn check_reach_destination_system(
    query: Query<(Entity, &GridPosition, &Walking), With<Agent>>,
    mut commands: Commands,
) {
    for (entity, position, walking) in &query {
        if position.eq(&walking.destination) {
            commands.entity(entity).remove::<Walking>();
        }
    }
}

#[derive(Default)]
struct OccupiedNow {
    pos: Vec<Entity>,
}

fn check_agent_pathfinding(
    query: Query<(Entity, &GridPosition, &Walking, &Agent)>,
    mut p_query: Query<&mut AgentPathfinding>,
    tile_query: Query<&Tile, Without<Occupied>>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
    mut occupied_now: Local<OccupiedNow>,
    occupied_positions_query: Query<&GridPosition, With<Occupied>>,
) {
    let dynamic_occupied_tiles: HashSet<GridPosition> =
        occupied_positions_query.iter().cloned().collect();

    for (agent_entity, agent_curr_position, walking, agent) in &query {
        if let Ok(mut pathfinding) = p_query.get_mut(agent.pathfinding_entity) {
            match pathfinding.as_mut() {
                AgentPathfinding::Nothing => {
                    pathfinding.start_path_calculation(agent_curr_position, &walking.destination);
                    UpdateAgentColor::calculating_path(&mut commands, agent_entity);
                }
                AgentPathfinding::Calculating(pathfinder) => {
                    if let Some(path) = pathfinder.get_path_if_finished() {
                        pathfinding.start_walking_path(path);
                        UpdateAgentColor::walking_path(&mut commands, agent_entity);
                    } else {
                        pathfinder.step(&spatial_idx, &dynamic_occupied_tiles);
                    }
                }
                AgentPathfinding::Ready(current_path) => {
                    if let AgentCurrentPathStatus::WaitingNextStep((step, retry)) =
                        &mut current_path.status
                    {
                        if *retry > 10 {
                            pathfinding
                                .start_path_calculation(agent_curr_position, &walking.destination);

                            UpdateAgentColor::calculating_path(&mut commands, agent_entity);

                            continue;
                        }

                        let is_last_step = current_path.path.len() == *step;

                        if is_last_step {
                            if let Some(last_step_position) = current_path.path.last() {
                                let reach_final_destination =
                                    last_step_position.eq(&walking.destination);

                                if reach_final_destination {
                                    pathfinding.reset();
                                } else {
                                    pathfinding.start_path_calculation(
                                        agent_curr_position,
                                        &walking.destination,
                                    );

                                    UpdateAgentColor::calculating_path(&mut commands, agent_entity);
                                }
                            }

                            continue;
                        }

                        let next_position = current_path.path.get(*step).expect("Out of bounds");

                        let tile_entity = spatial_idx
                            .get_entity(next_position.x, next_position.y)
                            .expect("next position do not exist");

                        if occupied_now.pos.contains(&tile_entity) {
                            *retry += 1;
                            continue;
                        }

                        if let Ok(_tile) = tile_query.get(tile_entity) {
                            occupied_now.pos.push(tile_entity.clone());
                            commands.entity(tile_entity.clone()).insert(Occupied);

                            if let Some(entity) =
                                spatial_idx.get_entity(agent_curr_position.x, agent_curr_position.y)
                            {
                                commands.entity(entity).remove::<Occupied>();
                            }

                            commands.trigger(UpdatePathfindingCurrentStep {
                                new_position: next_position.clone(),
                                entity: agent.pathfinding_entity,
                            });
                        } else {
                            *retry += 1;
                        }
                    }
                }
            }
        }
    }
    occupied_now.pos.clear();
}

#[derive(Event, Debug)]
struct UpdatePathfindingCurrentStep {
    entity: Entity,
    new_position: GridPosition,
}

fn update_pathfinding_curr_step(
    event: On<UpdatePathfindingCurrentStep>,
    mut p_query: Query<(&mut GridPosition, &mut Transform, &mut AgentPathfinding)>,
) {
    if let Ok((mut curr_position, mut transform, mut pathfinding)) = p_query.get_mut(event.entity) {
        match pathfinding.as_mut() {
            AgentPathfinding::Ready(curr_path) => {
                if let AgentCurrentPathStatus::WaitingNextStep((step, _)) = curr_path.status {
                    curr_path.status = AgentCurrentPathStatus::RunningStep(step);

                    curr_position.x = event.new_position.x;
                    curr_position.y = event.new_position.y;

                    let mut new_point =
                        Grid::grid_to_world(event.new_position.x, event.new_position.y);
                    new_point.z = PATHFINDER_Z_VALUE;
                    transform.translation = new_point;
                }
            }
            _ => {}
        }
    }
}

#[derive(Event, Debug)]
struct UpdateAgentColor {
    entity: Entity,
    color: Color,
}

impl UpdateAgentColor {
    pub fn calculating_path(commands: &mut Commands, agent_entity: Entity) {
        commands.trigger(UpdateAgentColor {
            entity: agent_entity,
            color: Color::linear_rgb(0.2, 1.0, 0.2),
        });
    }

    pub fn walking_path(commands: &mut Commands, agent_entity: Entity) {
        commands.trigger(UpdateAgentColor {
            entity: agent_entity,
            color: Color::linear_rgb(1.0, 0.2, 0.2),
        });
    }
}

fn on_update_agent_color(event: On<UpdateAgentColor>, mut p_query: Query<&mut Sprite>) {
    if let Ok(mut sprite) = p_query.get_mut(event.entity) {
        sprite.color = event.color;
    }
}

#[derive(Event, Debug)]
struct PathfindingFinishPathStep {
    entity: Entity,
}

fn pathfinding_finish_path_step(
    event: On<PathfindingFinishPathStep>,
    mut p_query: Query<&mut AgentPathfinding>,
) {
    if let Ok(mut pathfinding) = p_query.get_mut(event.entity) {
        match pathfinding.as_mut() {
            AgentPathfinding::Ready(curr_path) => {
                if let AgentCurrentPathStatus::RunningStep(step) = curr_path.status {
                    curr_path.status = AgentCurrentPathStatus::WaitingNextStep((step + 1, 0));
                }
            }
            _ => {}
        }
    }
}

#[derive(Event, Debug)]
struct UpdateAgentGridPosition {
    entity: Entity,
    new_position: GridPosition,
}

fn update_agent_position(
    event: On<UpdateAgentGridPosition>,
    mut query: Query<&mut GridPosition, With<Agent>>,
) {
    if let Ok(mut position) = query.get_mut(event.entity) {
        position.x = event.new_position.x;
        position.y = event.new_position.y;
    }
}

fn movement_agent(
    mut query: Query<(Entity, &GridPosition, &mut Transform, &Agent), With<Walking>>,
    p_query: Query<&GridPosition, With<AgentPathfinding>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, agent_position, mut transform, agent) in &mut query {
        if let Ok(pathfinding_position) = p_query.get(agent.pathfinding_entity) {
            if pathfinding_position.ne(agent_position) {
                let current_point = transform.translation;
                let mut target_point =
                    Grid::grid_to_world(pathfinding_position.x, pathfinding_position.y);

                target_point.z = AGENT_Z_VALUE;

                let to_target = target_point - current_point;
                let distance = to_target.length();
                let speed = 75.0;

                let step = speed * time.delta_secs();

                if step >= distance {
                    transform.translation = target_point;

                    commands.trigger(UpdateAgentGridPosition {
                        entity,
                        new_position: pathfinding_position.clone(),
                    });

                    commands.trigger(PathfindingFinishPathStep {
                        entity: agent.pathfinding_entity,
                    });
                } else {
                    let direction = to_target / distance;
                    transform.translation += direction * step;
                }
            }
        }
    }
}
