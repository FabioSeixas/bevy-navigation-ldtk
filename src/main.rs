mod constants;
mod pathfinder;
mod spatial_idx;
mod world;

use std::collections::HashSet;

use bevy::{color::palettes::css::*, gizmos::config::GizmoConfig, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use constants::*;
use pathfinder::*;
use spatial_idx::*;
use world::*;

#[derive(Resource)]
struct SpawnAgentTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LdtkPlugin)
        .init_resource::<GizmoConfigStore>()
        .insert_resource(LevelSelection::index(0))
        .init_resource::<SpatialIndex>()
        .insert_resource(SpawnAgentTimer(Timer::from_seconds(2.0, TimerMode::Once)))
        .add_systems(PreStartup, (setup_camera, spawn_grid).chain())
        // .add_systems(Startup, (setup_camera, spawn_grid).chain())
        .add_observer(on_add_tile)
        .add_observer(on_add_tile_enum_tags)
        .add_observer(update_pathfinding_curr_step)
        .add_observer(pathfinding_finish_path_step)
        .add_observer(update_agent_position)
        .add_observer(on_update_agent_color)
        .add_systems(
            Update,
            (
                mark_destination_on_map,
                draw_tile_gizmos,
                on_disocuppied,
                define_destination_system,
                check_reach_destination_system,
                movement_agent,
                check_agent_pathfinding,
                mouse_click_world_pos,
                spawn_agent_system,
                toggle_gizmos,
                // debug,
            ),
        )
        .run();
}

fn toggle_gizmos(mut config_store: ResMut<GizmoConfigStore>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyG) {
        let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
        config.enabled = !config.enabled;
    }
}

fn debug(query: Query<(&GridPosition, &Transform)>) {
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
            scale: 1.2,

            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(512.0, 512.0, 0.0),
    ));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("proj.ldtk").into(),

        ..Default::default()
    });
}

fn spawn_grid(mut commands: Commands) {
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let pos = Grid::grid_to_world(x, y);

            commands.spawn((
                Transform::from_translation(pos),
                Tile { x, y },
                GridPosition { x, y },
            ));
        }
    }
}

#[derive(Component, Default)]
enum AgentPathfinding {
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
struct AgentCurrentPath {
    path: Vec<GridPosition>,
    status: AgentCurrentPathStatus,
}

#[derive(Debug, PartialEq, Eq)]
enum AgentCurrentPathStatus {
    WaitingNextStep((usize, usize)), // (step_idx, retry_count)

    RunningStep(usize),
}

fn spawn_agent_system(
    mut commands: Commands,

    time: Res<Time>,

    mut timer: Option<ResMut<SpawnAgentTimer>>,

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
                // Check if it's not a Wall or Door

                if tile_data.tile_type != TileType::Wall && tile_data.tile_type != TileType::Door {
                    // And also check if it's not dynamically occupied

                    if let Ok(_) = tile_query.get(tile_data.entity) {
                        chosen_destination_pos = Some(pos);
                    }
                }
            }
        }

        // Once a valid destination is found

        if let Some(destination_pos) = chosen_destination_pos {
            commands.entity(agent_entity).insert(Walking {
                destination: destination_pos,
            });
        }
    }
}

fn mark_destination_on_map(query: Query<&Walking, With<Agent>>, mut gizmos: Gizmos) {
    for walking in &query {
        let pos = Grid::grid_to_world(walking.destination.x, walking.destination.y);

        let half_tile: f32 = TILE_SIZE / 2.;

        gizmos.line_2d(
            Vec2 {
                x: pos.x - half_tile,

                y: pos.y - half_tile,
            },
            Vec2 {
                x: pos.x + half_tile,

                y: pos.y + half_tile,
            },
            RED,
        );

        gizmos.line_2d(
            Vec2 {
                x: pos.x - half_tile,

                y: pos.y + half_tile,
            },
            Vec2 {
                x: pos.x + half_tile,

                y: pos.y - half_tile,
            },
            RED,
        );
    }
}

fn draw_tile_gizmos(
    spatial_idx: Res<SpatialIndex>,

    occupied_query: Query<&GridPosition, With<Occupied>>,

    mut gizmos: Gizmos,
) {
    // First, draw gizmos for TileTypes

    for (coords_tuple, tile_data) in &spatial_idx.map {
        let pos = Grid::grid_to_world(coords_tuple.0, coords_tuple.1);

        let color: Option<Srgba> = match tile_data.tile_type {
            TileType::Inside => Some(GREEN.into()),

            TileType::Wall => Some(GRAY.into()),

            TileType::Door => Some(YELLOW.into()),

            TileType::Outside => None, // Don't draw anything for Outside
        };

        if let Some(color) = color {
            gizmos.rect_2d(
                pos.truncate(),
                // 0.0,
                Vec2::splat(TILE_SIZE - 4.0), // A bit smaller than the tile
                color,
            );
        }
    }

    // Then, draw over them for Occupied tiles

    for coords in occupied_query.iter() {
        let pos = Grid::grid_to_world(coords.x, coords.y);

        gizmos.circle_2d(pos.truncate(), TILE_SIZE / 4.0, BLUE);
    }
}

fn check_reach_destination_system(
    query: Query<(Entity, &GridPosition, &Walking), With<Agent>>,
    mut commands: Commands,
) {
    for (entity, position, walking) in &query {
        // println!("check_reach_destination_system");
        if position.eq(&walking.destination) {
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
                    // println!("new pathfinding position: {:?}", curr_position);
                    // println!("new pathfinding translation: {:?}\n", new_point);
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
        // println!("update_agent_position: {:?}", event.new_position);
        position.x = event.new_position.x;
        position.y = event.new_position.y;
    }
}

fn on_disocuppied(mut removed: RemovedComponents<Occupied>, query: Query<&Tile>) {
    for entity in removed.read() {
        if let Ok(tile) = query.get(entity) {
            // println!("\non_disocuppied: removed from tile: {:?}", tile);
        }
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
                        new_position: pathfinding_position.clone(),
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
