mod constants;
mod spatial_idx;

use bevy::{math::ops::abs, prelude::*};
use constants::*;
use rand::Rng;
use spatial_idx::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Grid::new(GRID_WIDTH, GRID_HEIGHT))
        .init_resource::<SpatialIndex>()
        .add_systems(Startup, (spawn_grid, setup_camera))
        .add_systems(PostStartup, spawn_agent)
        .add_observer(on_add_tile)
        .add_observer(update_agent_pathfinding)
        .add_observer(update_agent_position)
        .add_systems(
            Update,
            (
                define_destination_system,
                check_reach_destination_system,
                movement_agent,
                check_agent_pathfinding,
            ),
        )
        .run();
}

/// A grid resource
#[derive(Resource)]
struct Grid {
    width: i32,
    height: i32,
}

impl Grid {
    fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Convert grid coordinates → world coordinates (Vec3)
    fn grid_to_world(x: i32, y: i32) -> Vec3 {
        let middle_x = GRID_WIDTH / 2;
        let middle_y = GRID_HEIGHT / 2;
        Vec3::new(
            (x - middle_x) as f32 * TILE_SIZE,
            (y - middle_y) as f32 * TILE_SIZE,
            0.0,
        )
    }

    fn get_random_position(&self) -> GridPosition {
        let mut rnd = rand::thread_rng();
        GridPosition {
            x: rnd.gen_range(0..self.width),
            y: rnd.gen_range(0..self.height),
        }
    }
}

/// Position on the grid
#[derive(Component, Debug, PartialEq, Eq, Clone)]
struct GridPosition {
    x: i32,
    y: i32,
}

/// Marker for the agent
#[derive(Component)]
struct Agent {
    pathfinding_entity: Entity,
}

/// Marker for the Pathfinding Entity
#[derive(Component)]
struct AgentPathfinding;

#[derive(Component)]
struct Walking {
    destination: GridPosition,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_grid(mut commands: Commands, grid: Res<Grid>) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let odd_x = x % 2 == 0;
            let odd_y = y % 2 == 0;

            let mut pos = Grid::grid_to_world(x, y);
            pos.z = -1.;

            if odd_x && odd_y {
                commands.spawn((
                    Sprite {
                        color: Color::linear_rgb(0.20, 0.20, 0.80),
                        custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)), // little gap
                        ..default()
                    },
                    Transform::from_translation(pos),
                    Tile { x, y },
                    Occupied,
                ));
            } else {
                commands.spawn((
                    Sprite {
                        color: Color::linear_rgb(0.15, 0.15, 0.15),
                        custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)), // little gap
                        ..default()
                    },
                    Transform::from_translation(pos),
                    Tile { x, y },
                ));
            }
        }
    }

    // println!("Grid spawned.");
}

fn spawn_agent(mut commands: Commands) {
    for i in 0..AGENTS_COUNT {
        let x = GRID_WIDTH / 2;
        let y = GRID_HEIGHT / 2;

        let pathfinding_entity = commands
            .spawn((
                AgentPathfinding,
                GridPosition { x, y },
                Sprite {
                    color: Color::linear_rgb(1.0, 1.2, 1.2),
                    custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)),
                    ..default()
                },
                Transform::from_translation(Grid::grid_to_world(x, y)),
            ))
            .id();

        commands.spawn((
            Agent { pathfinding_entity },
            GridPosition { x, y },
            Sprite {
                color: Color::linear_rgb(1.0, 0.2, 0.2),
                custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)),
                ..default()
            },
            Transform::from_translation(Grid::grid_to_world(x, y)),
        ));
    }
}

/// Marker for the Pathfinding Entity
#[derive(Component)]
struct Occupied;

fn define_destination_system(
    mut query: Query<Entity, (Without<Walking>, With<Agent>)>,
    tile_query: Query<&Tile, Without<Occupied>>,
    grid: Res<Grid>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
) {
    for entity in &mut query {
        let pos = grid.get_random_position();
        if let Ok(_) = tile_query.get(spatial_idx.get_entity(pos.x, pos.y)) {
            commands.entity(entity).insert(Walking { destination: pos });
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
    query: Query<(&GridPosition, &Walking, &Agent)>,
    p_query: Query<&GridPosition, With<AgentPathfinding>>,
    tile_query: Query<&Tile, Without<Occupied>>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
    mut occupied_now: Local<OccupiedNow>,
) {
    for (agent_curr_position, walking, agent) in &query {
        if let Ok(pathfinding_curr_position) = p_query.get(agent.pathfinding_entity) {
            // println!(
            //     "check_agent_pathfinding: agent position {:?}",
            //     agent_position
            // );
            // println!(
            //     "check_agent_pathfinding: pathfinding position {:?}",
            //     position
            // );

            if pathfinding_curr_position.eq(&agent_curr_position) {
                let current_point =
                    Grid::grid_to_world(agent_curr_position.x, agent_curr_position.y);
                let destination_point =
                    Grid::grid_to_world(walking.destination.x, walking.destination.y);

                println!("\ndesired_point: {:?}", walking.destination);
                let normalized = (destination_point - current_point).normalize();
                println!("normalized: {}", normalized);

                if normalized.is_nan() {
                    continue;
                }

                let mut new_position = GridPosition {
                    y: pathfinding_curr_position.y,
                    x: pathfinding_curr_position.x,
                };

                // Get the next logical path
                if abs(normalized.x) > abs(normalized.y) {
                    if normalized.x > 0. {
                        new_position.x += 1;
                    } else {
                        new_position.x -= 1;
                    }
                } else {
                    if normalized.y > 0. {
                        new_position.y += 1;
                    } else {
                        new_position.y -= 1;
                    }
                }
                println!("current pos {:?}", agent_curr_position);
                println!("new pos {:?}", new_position);

                let mut maybe_next_path: Option<(GridPosition, f32, Entity)> = None;

                let current_tile_entity = spatial_idx.get_entity(new_position.x, new_position.y);

                // If the next logical path is not available, calculate an alternative
                if occupied_now.pos.contains(&current_tile_entity)
                    || tile_query.get(current_tile_entity).is_err()
                {
                    for candidate_tile_entity in
                        spatial_idx.get_nearby(agent_curr_position.x, agent_curr_position.y)
                    {
                        // Already tried
                        if candidate_tile_entity.eq(&current_tile_entity) {
                            continue;
                        }

                        // Avoid two Agents try the same tile at the same frame
                        if occupied_now.pos.contains(&candidate_tile_entity) {
                            continue;
                        }

                        println!("current next path {:?}", maybe_next_path);
                        println!("check candidate {:?}", candidate_tile_entity);

                        if let Ok(free_tile) = tile_query.get(candidate_tile_entity) {
                            let free_tile_point = Grid::grid_to_world(free_tile.x, free_tile.y);

                            let distance: f32 = free_tile_point.distance(destination_point);

                            if let Some((_next_path_candidate, curr_distance, _entity)) =
                                maybe_next_path.as_ref()
                            {
                                if *curr_distance > distance {
                                    maybe_next_path = Some((
                                        GridPosition {
                                            x: free_tile.x,
                                            y: free_tile.y,
                                        },
                                        distance,
                                        candidate_tile_entity,
                                    ));
                                }
                            } else {
                                maybe_next_path = Some((
                                    GridPosition {
                                        x: free_tile.x,
                                        y: free_tile.y,
                                    },
                                    distance,
                                    candidate_tile_entity,
                                ));
                            }
                        }
                    }

                    if let Some((new_path, _distance, tile_entity)) = maybe_next_path.as_ref() {
                        new_position.x = new_path.x;
                        new_position.y = new_path.y;

                        occupied_now.pos.push(tile_entity.clone());

                        commands.entity(tile_entity.clone()).insert(Occupied);

                        commands
                            .entity(
                                spatial_idx
                                    .get_entity(agent_curr_position.x, agent_curr_position.y),
                            )
                            .remove::<Occupied>();
                    }
                } else {
                    maybe_next_path = Some((new_position, 0.0, Entity::PLACEHOLDER));
                }

                if let Some((new_position, _, _)) = maybe_next_path.take() {
                    commands.trigger(UpdatePathfinding {
                        new_position,
                        entity: agent.pathfinding_entity,
                    });
                }
            }
        }
    }

    occupied_now.pos.clear();
}

#[derive(Event, Debug)]
struct UpdatePathfinding {
    entity: Entity,
    new_position: GridPosition,
}

fn update_agent_pathfinding(
    event: On<UpdatePathfinding>,
    mut p_query: Query<(&mut GridPosition, &mut Transform), With<AgentPathfinding>>,
) {
    if let Ok((mut position, mut transform)) = p_query.get_mut(event.entity) {
        // println!(
        //     "update_agent_pathfinding: new position {:?}",
        //     trigger.new_position
        // );
        position.x = event.new_position.x;
        position.y = event.new_position.y;

        let new_point = Grid::grid_to_world(event.new_position.x, event.new_position.y);
        // println!("update_agent_pathfinding: new point {:?}", new_point);
        transform.translation = new_point;
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
    for (entity, position, mut transform, agent) in &mut query {
        if let Ok(pathfinding_position) = p_query.get(agent.pathfinding_entity) {
            if pathfinding_position.ne(position) {
                let current_point = transform.translation;
                let previous_point = Grid::grid_to_world(position.x, position.y);
                let target_point =
                    Grid::grid_to_world(pathfinding_position.x, pathfinding_position.y);

                let to_target = target_point - current_point;
                let distance = to_target.length();
                let speed = 75.0;

                let step = speed * time.delta_secs();

                // println!("\nmovement_agent: current_point: {}", current_point);
                // println!("movement_agent: previous_point: {}", previous_point);
                // println!("movement_agent: target_point: {}", target_point);
                // println!("movement_agent: distance to target: {}", distance);
                // println!("movement_agent: step: {}\n", step);

                if step >= distance {
                    transform.translation = target_point;

                    commands.trigger(UpdateAgentGridPosition {
                        entity,
                        new_position: pathfinding_position.clone(),
                    });
                } else {
                    let direction = to_target / distance;
                    transform.translation += direction * step;
                }
            }
        }
    }
}
