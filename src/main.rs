mod constants;
mod pathfinder;
mod spatial_idx;

use std::collections::HashSet;

use bevy::{color::palettes::css::*, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use constants::*;
use pathfinder::*;
use rand::Rng;
use spatial_idx::*;

trait ConvertableToGridPosition {
    fn to_grid_position(&self) -> GridPosition;
}

impl ConvertableToGridPosition for GridCoords {
    fn to_grid_position(&self) -> GridPosition {
        GridPosition {
            x: self.x,
            y: self.y,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LdtkPlugin)
        .insert_resource(LevelSelection::index(0))
        .init_resource::<SpatialIndex>()
        .add_systems(Startup, setup_camera)
        .add_systems(PostStartup, spawn_agent)
        .add_observer(on_add_tile)
        .add_observer(on_add_tile_enum_tags)
        .add_observer(update_pathfinding_curr_step)
        .add_observer(pathfinding_finish_path_step)
        .add_observer(update_agent_position)
        .add_observer(update_agent_color)
        .add_systems(
            Update,
            (
                mark_destination_on_map,
                mark_occupied_on_map,
                on_disocuppied,
                define_destination_system,
                check_reach_destination_system,
                movement_agent,
                check_agent_pathfinding,
                mouse_click_world_pos,
                // debug,
            ),
        )
        .run();
}

fn debug(query: Query<(&GridCoords, &Transform)>) {
    for (coords, transform) in query {
        if coords.x == 0 {
            dbg!(coords);
            dbg!(transform.translation);
        }
    }
}

fn mouse_click_world_pos(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
) {
    // detect right click
    if buttons.just_pressed(MouseButton::Right) {
        let window = windows.single().unwrap();

        // get the cursor window position
        if let Some(screen_pos) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                // convert window position -> world coordinates
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
                    let grid_position = Grid::world_to_grid(world_pos);
                    if let Some(entity) = spatial_idx.get_entity(grid_position.x, grid_position.y) {
                        commands.entity(entity).insert((
                            Sprite {
                                color: Color::linear_rgb(0.20, 0.20, 0.80),
                                custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)), // little gap
                                ..default()
                            },
                            Occupied,
                        ));
                        // println!("\n SET AS OCCUPIED {:?}\n", grid_position);
                    }
                }
            }
        }
    }
}

struct Grid;

impl Grid {
    /// Convert grid coordinates → world coordinates (Vec3)
    fn grid_to_world(x: i32, y: i32) -> Vec3 {
        Vec3::new(
            x as f32 * TILE_SIZE + (TILE_SIZE / 2.),
            y as f32 * TILE_SIZE + (TILE_SIZE / 2.),
            0.0,
        )
    }

    fn world_to_grid(pos: Vec2) -> GridPosition {
        GridPosition {
            x: (pos.x / TILE_SIZE) as i32,
            y: (pos.y / TILE_SIZE) as i32,
        }
    }

    fn get_random_position() -> GridPosition {
        let mut rnd = rand::thread_rng();
        GridPosition {
            x: rnd.gen_range(0..GRID_WIDTH),
            y: rnd.gen_range(0..GRID_HEIGHT),
        }
    }

    fn coords_to_grid_position(c: GridCoords) -> GridPosition {
        GridPosition { x: c.x, y: c.y }
    }

    fn grid_position_to_coords(gp: GridPosition) -> GridCoords {
        GridCoords { x: gp.x, y: gp.y }
    }
}

/// Position on the grid
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct GridPosition {
    x: i32,
    y: i32,
}

/// Marker for the agent
#[derive(Component)]
struct Agent {
    pathfinding_entity: Entity,
}

#[derive(Component)]
struct Walking {
    destination: GridPosition,
}

fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(1280.0 / 4.0, 720.0 / 4.0, 0.0),
    ));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("proj.ldtk").into(),
        ..Default::default()
    });
}

#[derive(Component, Default)]
enum AgentPathfinding {
    #[default]
    Nothing,
    Calculating(Pathfinder),
    Ready(AgentCurrentPath),
}

#[derive(Debug)]
struct AgentCurrentPath {
    path: Vec<GridPosition>,
    status: AgentCurrentPathStatus,
}

#[derive(Debug, PartialEq, Eq)]
enum AgentCurrentPathStatus {
    WaitingNextStep((usize, usize)), // (step_idx, retry_count)
    RunningStep(usize),
}

fn spawn_agent(mut commands: Commands) {
    for _ in 0..AGENTS_COUNT {
        let grid_pos = Grid::get_random_position();
        let pos = Grid::grid_to_world(grid_pos.x, grid_pos.y);

        let pathfinding_entity = commands
            .spawn((
                AgentPathfinding::default(),
                Grid::grid_position_to_coords(grid_pos.clone()),
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
            Grid::grid_position_to_coords(grid_pos),
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

#[derive(Component)]
struct Occupied;

fn define_destination_system(
    mut query: Query<Entity, (Without<Walking>, With<Agent>)>,
    tile_query: Query<&TilemapId, Without<Occupied>>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
) {
    for agent_entity in &mut query {
        let pos = Grid::get_random_position();
        // println!("define_destination_system: position {:?}", pos);
        if let Some(entity) = spatial_idx.get_entity(pos.x, pos.y) {
            if let Ok(_) = tile_query.get(entity) {
                commands
                    .entity(agent_entity)
                    .insert(Walking { destination: pos });
            }
        }
    }
}

fn mark_destination_on_map(query: Query<&Walking, With<Agent>>, mut gizmos: Gizmos) {
    for walking in &query {
        let pos = Grid::grid_to_world(walking.destination.x, walking.destination.y);

        let half_tile: f32 = TILE_SIZE / 2.;

        gizmos.line_2d(
            Vec2 { x: pos.x - half_tile, y: pos.y - half_tile },
            Vec2 {
                x: pos.x + half_tile,
                y: pos.y + half_tile
            },
            RED,
        );

        gizmos.line_2d(
            Vec2 { x: pos.x - half_tile, y: pos.y + half_tile },
            Vec2 {
                x: pos.x + half_tile,
                y: pos.y - half_tile
            },
            RED,
        );
    }
}

fn mark_occupied_on_map(query: Query<&GridCoords, With<Occupied>>, mut gizmos: Gizmos) {
    for coords in &query {
        let pos = Grid::grid_to_world(coords.x, coords.y);

        let half_tile: f32 = TILE_SIZE / 2.;

        gizmos.line_2d(
            Vec2 { x: pos.x - half_tile, y: pos.y - half_tile },
            Vec2 {
                x: pos.x + half_tile,
                y: pos.y + half_tile
            },
            BLUE,
        );

        gizmos.line_2d(
            Vec2 { x: pos.x - half_tile, y: pos.y + half_tile },
            Vec2 {
                x: pos.x + half_tile,
                y: pos.y - half_tile
            },
            BLUE,
        );
    }
}

fn check_reach_destination_system(
    query: Query<(Entity, &GridCoords, &Walking), With<Agent>>,
    mut commands: Commands,
) {
    for (entity, position, walking) in &query {
        // println!("check_reach_destination_system");
        if position.to_grid_position().eq(&walking.destination) {
            // println!("destination reached");
            commands.entity(entity).remove::<Walking>();
        }
    }
}

#[derive(Default)]
struct OccupiedNow {
    pos: Vec<Entity>,
}

fn check_agent_pathfinding(
    query: Query<(Entity, &GridCoords, &Walking, &Agent)>,
    mut p_query: Query<&mut AgentPathfinding>,
    tile_query: Query<&TilemapId, Without<Occupied>>,
    spatial_idx: Res<SpatialIndex>,
    mut commands: Commands,
    mut occupied_now: Local<OccupiedNow>,
) {
    for (agent_entity, agent_grid_coords, walking, agent) in &query {
        let agent_curr_position = agent_grid_coords.to_grid_position();
        if let Ok(mut pathfinding) = p_query.get_mut(agent.pathfinding_entity) {
            match pathfinding.as_mut() {
                AgentPathfinding::Nothing => {
                    *pathfinding = AgentPathfinding::Calculating(Pathfinder::new(
                        &agent_curr_position,
                        &walking.destination,
                    ));

                    commands.trigger(UpdateAgentColor {
                        entity: agent_entity,
                        color: Color::linear_rgb(0.2, 1.0, 0.2),
                    });
                }
                AgentPathfinding::Calculating(pathfinder) => {
                    if let Some(path) = pathfinder.get_path_if_finished() {
                        *pathfinding = AgentPathfinding::Ready(AgentCurrentPath {
                            path,
                            status: AgentCurrentPathStatus::WaitingNextStep((0, 0)),
                        });
                        commands.trigger(UpdateAgentColor {
                            entity: agent_entity,
                            color: Color::linear_rgb(1.0, 0.2, 0.2),
                        });
                        continue;
                    }

                    if let Some(pos) = pathfinder.get_current_node_position() {
                        let mut unavailable_nearby_positions: HashSet<GridPosition> =
                            HashSet::new();

                        for (entity, grid_position) in spatial_idx.get_nearby(pos.x, pos.y) {
                            if occupied_now.pos.contains(&entity) {
                                // println!("occupied_now: {:?}", grid_position);
                                unavailable_nearby_positions.insert(grid_position.clone());
                            } else if tile_query.get(entity).is_err() {
                                // println!("tile occupied: {:?}", grid_position);
                                unavailable_nearby_positions.insert(grid_position.clone());
                            }
                        }

                        pathfinder.step(unavailable_nearby_positions);
                    }
                }
                AgentPathfinding::Ready(current_path) => {
                    if let AgentCurrentPathStatus::WaitingNextStep((step, retry)) =
                        &mut current_path.status
                    {
                        // println!("curr_path: {:?}", current_path);

                        if *retry > 10 {
                            *pathfinding = AgentPathfinding::Calculating(Pathfinder::new(
                                &agent_curr_position,
                                &walking.destination,
                            ));

                            commands.trigger(UpdateAgentColor {
                                entity: agent_entity,
                                color: Color::linear_rgb(0.2, 1.0, 0.2),
                            });

                            continue;
                        }

                        if current_path.path.len() == *step {
                            // println!("reach destination");
                            if let Some(last_step_position) = current_path.path.last() {
                                if last_step_position.eq(&walking.destination) {
                                    *pathfinding = AgentPathfinding::Nothing;
                                } else {
                                    *pathfinding = AgentPathfinding::Calculating(Pathfinder::new(
                                        &agent_curr_position,
                                        &walking.destination,
                                    ));

                                    commands.trigger(UpdateAgentColor {
                                        entity: agent_entity,
                                        color: Color::linear_rgb(0.2, 1.0, 0.2),
                                    });
                                }
                            }

                            // free previous tile
                            if let Some(entity) =
                                spatial_idx.get_entity(agent_curr_position.x, agent_curr_position.y)
                            {
                                commands.entity(entity).remove::<Occupied>();
                            }

                            continue;
                        }

                        let next_position = current_path.path.get(*step).expect("Out of bounds");

                        // println!("next_position: {:?}", next_position);

                        let tile_entity = spatial_idx
                            .get_entity(next_position.x, next_position.y)
                            .expect("next position do not exist");

                        // Avoid two Agents try the same tile at the same frame
                        if occupied_now.pos.contains(&tile_entity) {
                            *retry += 1;

                            // println!("Next step tile not available {:?}", next_position);
                            // println!("retry {}", retry);

                            continue;
                        }

                        if let Ok(_tile) = tile_query.get(tile_entity) {
                            occupied_now.pos.push(tile_entity.clone());

                            // mark next tile as occupied
                            commands.entity(tile_entity.clone()).insert(Occupied);

                            // free previous tile
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
                            // println!("Next step tile not available {:?}", next_position);
                            // println!("retry {}", retry);
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
    mut p_query: Query<(&mut GridCoords, &mut Transform, &mut AgentPathfinding)>,
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
                    println!("new pathfinding position: {:?}", curr_position);
                    println!("new pathfinding translation: {:?}\n", new_point);
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

fn update_agent_color(event: On<UpdateAgentColor>, mut p_query: Query<&mut Sprite>) {
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
    mut query: Query<&mut GridCoords, With<Agent>>,
) {
    if let Ok(mut position) = query.get_mut(event.entity) {
        // println!("update_agent_position: {:?}", event.new_position);
        position.x = event.new_position.x;
        position.y = event.new_position.y;
    }
}

fn on_disocuppied(mut removed: RemovedComponents<Occupied>, query: Query<&TilemapId>) {
    for entity in removed.read() {
        if let Ok(tile) = query.get(entity) {
            // println!("\non_disocuppied: removed from tile: {:?}", tile);
        }
    }
}

fn movement_agent(
    mut query: Query<(Entity, &GridCoords, &mut Transform, &Agent), With<Walking>>,
    p_query: Query<&GridCoords, With<AgentPathfinding>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, agent_position, mut transform, agent) in &mut query {
        if let Ok(pathfinding_position) = p_query.get(agent.pathfinding_entity) {
            if pathfinding_position.ne(agent_position) {
                let current_point = transform.translation;
                // let previous_point = Grid::grid_to_world(agent_position.x, agent_position.y);
                let mut target_point =
                    Grid::grid_to_world(pathfinding_position.x, pathfinding_position.y);

                target_point.z = AGENT_Z_VALUE;

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

                    // println!(
                    //     "will update agent grid position to {:?}",
                    //     pathfinding_position
                    // );

                    commands.trigger(UpdateAgentGridPosition {
                        entity,
                        new_position: pathfinding_position.to_grid_position().clone(),
                    });

                    commands.trigger(PathfindingFinishPathStep {
                        entity: agent.pathfinding_entity,
                    });
                } else {
                    let direction = to_target / distance;
                    transform.translation += direction * step;
                }

                // println!("agent new translation {}", transform.translation);
            }
        }
    }
}
