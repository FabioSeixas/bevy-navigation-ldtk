use std::collections::HashSet;

use bevy::{gizmos::config::DefaultGizmoConfigGroup, prelude::*, sprite::Anchor};

use crate::{
    animation::{AnimationDirection, AnimationTimer, CharacterAnimations, CharacterSpriteSheet},
    constants::*,
    events::{AgentEnteredTile, AgentLeftTile},
    pathfinder::Pathfinder,
    walk::Walking,
    world::{components::*, grid::*, spatial_idx::*},
};

#[derive(Component)]
pub struct AgentDebugColor(pub Color);

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnAgentTimer(Timer::from_seconds(2.0, TimerMode::Once)))
            .add_observer(update_pathfinding_curr_step)
            .add_observer(pathfinding_finish_path_step)
            .add_observer(update_agent_position)
            .add_systems(
                Update,
                (
                    update_agents_needs_system,
                    movement_agent,
                    check_agent_pathfinding,
                    spawn_agent_system,
                    toggle_pathfinding_ui_visibility,
                    update_agent_colors_based_on_gizmos,
                ),
            );
    }
}

#[derive(Resource)]
struct SpawnAgentTimer(Timer);

fn spawn_agent_system(
    mut commands: Commands,
    time: Res<Time>,
    timer: Option<ResMut<SpawnAgentTimer>>,
    query: Query<&GridPosition, (With<Tile>, Without<Occupied>)>,
    spatial_idx: Res<SpatialIndex>,
    character_sprite_sheet: Res<CharacterSpriteSheet>,
    animations: Res<CharacterAnimations>,
) {
    if let Some(mut timer) = timer {
        if timer.0.tick(time.delta()).just_finished() {
            for _ in 0..AGENTS_COUNT {
                let mut done = false;
                while !done {
                    let grid_pos = Grid::get_random_position();
                    if let Some(tile_data) = spatial_idx.map.get(&(grid_pos.x, grid_pos.y)) {
                        if tile_data.is_outside() {
                            if let Ok(_) = query.get(tile_data.entity) {
                                done = true;
                                let pos = Grid::grid_to_world(grid_pos.x, grid_pos.y);
                                let pathfinding_entity = commands
                                    .spawn((
                                        AgentPathfinding::default(),
                                        grid_pos.clone(),
                                        Sprite {
                                            color: Color::srgb(1.0, 1.2, 1.2),
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
                                    Agent {
                                        pathfinding_entity,
                                        hungry: 0.,
                                    },
                                    grid_pos,
                                    Sprite::from_atlas_image(
                                        character_sprite_sheet.texture.clone(),
                                        TextureAtlas {
                                            layout: character_sprite_sheet
                                                .texture_atlas_layout
                                                .clone(),
                                            index: animations.walk_down.first,
                                        },
                                    ),
                                    Anchor::BOTTOM_CENTER,
                                    Transform::from_translation(Vec3 {
                                        x: pos.x,
                                        y: pos.y,
                                        z: AGENT_Z_VALUE,
                                    })
                                    .with_scale(Vec3::splat(0.8)),
                                    animations.walk_down,
                                    AnimationDirection::Down,
                                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
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

/// Marker for the agent
#[derive(Component)]
pub struct Agent {
    pathfinding_entity: Entity,
    hungry: f32,
}

impl Agent {
    pub fn is_hungry(&self) -> bool {
        self.hungry > 1000.
    }

    pub fn fill_hungry(&mut self) {
        self.hungry = 0.
    }
}

fn update_agents_needs_system(mut q_agents: Query<&mut Agent>) {
    for mut agent in &mut q_agents {
        agent.hungry += 1.;
    }
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
                            commands.trigger(AgentEnteredTile {
                                entity: tile_entity,
                            });

                            if let Some(entity) =
                                spatial_idx.get_entity(agent_curr_position.x, agent_curr_position.y)
                            {
                                commands.trigger(AgentLeftTile { entity });
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

struct UpdateAgentColor;
impl UpdateAgentColor {
    pub fn calculating_path(commands: &mut Commands, agent_entity: Entity) {
        commands
            .entity(agent_entity)
            .insert(AgentDebugColor(Color::srgb(0.2, 1.0, 0.2)));
    }

    pub fn walking_path(commands: &mut Commands, agent_entity: Entity) {
        commands
            .entity(agent_entity)
            .insert(AgentDebugColor(Color::srgb(1.0, 0.2, 0.2)));
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
    query: Query<
        (
            Entity,
            &GridPosition,
            &mut Transform,
            &mut AnimationDirection,
            &mut AnimationTimer,
            &mut Sprite,
            &Agent,
        ),
        With<Walking>,
    >,
    p_query: Query<&GridPosition, With<AgentPathfinding>>,
    time: Res<Time>,
    mut commands: Commands,
    animations: Res<CharacterAnimations>,
) {
    for (entity, agent_position, mut transform, mut anim_direction, mut timer, mut sprite, agent) in
        query
    {
        if let Ok(pathfinding_position) = p_query.get(agent.pathfinding_entity) {
            if pathfinding_position.ne(agent_position) {
                timer.unpause();

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
                    let direction_vec = to_target.normalize();

                    let new_direction = if direction_vec.x.abs() > direction_vec.y.abs() {
                        if direction_vec.x > 0.0 {
                            AnimationDirection::Right
                        } else {
                            AnimationDirection::Left
                        }
                    } else {
                        if direction_vec.y > 0.0 {
                            AnimationDirection::Up
                        } else {
                            AnimationDirection::Down
                        }
                    };

                    if *anim_direction != new_direction {
                        *anim_direction = new_direction;
                    }

                    transform.translation += direction_vec * step;
                }
            } else {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    timer.pause();
                    match *anim_direction {
                        AnimationDirection::Up => atlas.index = animations.walk_up.first,
                        AnimationDirection::Down => atlas.index = animations.walk_down.first,
                        AnimationDirection::Left => atlas.index = animations.walk_left.first,
                        AnimationDirection::Right => atlas.index = animations.walk_right.first,
                    }
                }
            }
        }
    }
}

fn toggle_pathfinding_ui_visibility(
    config_store: Res<GizmoConfigStore>,
    mut query: Query<&mut Visibility, With<AgentPathfinding>>,
) {
    let (config, _) = config_store.config::<DefaultGizmoConfigGroup>();
    for mut visibility in query.iter_mut() {
        if config.enabled {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn update_agent_colors_based_on_gizmos(
    config_store: Res<GizmoConfigStore>,
    mut query: Query<(&mut Sprite, &AgentDebugColor), With<Agent>>,
) {
    let (config, _) = config_store.config::<DefaultGizmoConfigGroup>();
    for (mut sprite, debug_color) in query.iter_mut() {
        if config.enabled {
            sprite.color = debug_color.0;
        } else {
            sprite.color = Color::WHITE;
        }
    }
}
