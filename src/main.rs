mod agent;
mod constants;
mod pathfinder;
mod roof;
mod spatial_idx;
mod world;

use agent::{Agent, AgentPlugin, Walking};
use bevy::{color::palettes::css::*, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TilePos};
use constants::*;
use roof::RoofPlugin;
use spatial_idx::*;
use world::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LdtkPlugin)
        .add_plugins(AgentPlugin)
        .add_plugins(RoofPlugin)
        .init_resource::<GizmoConfigStore>()
        .insert_resource(LevelSelection::index(0))
        .init_resource::<SpatialIndex>()
        .add_systems(PreStartup, (setup_camera, spawn_grid).chain())
        .add_observer(on_add_tile)
        .add_observer(on_add_tile_enum_tags)
        .add_systems(
            Update,
            (
                mark_destination_on_map,
                draw_tile_gizmos,
                on_disocuppied,
                mouse_click_world_pos,
                toggle_gizmos,
                // debug
            ),
        )
        .run();
}

// TilemapBundle
fn debug(query: Query<(Entity, &TilePos, &Transform, &TileColor)>) {
    for (entity, tile_pos, transform, tile_color) in query {
            dbg!(entity);
            dbg!(tile_color);
            dbg!(tile_pos);
            dbg!(transform.translation);
            println!("-------------------");
    }
}

fn toggle_gizmos(mut config_store: ResMut<GizmoConfigStore>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyG) {
        let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
        config.enabled = !config.enabled;
    }
}

// fn debug(query: Query<(&GridPosition, &Transform)>) {
//     for (coords, transform) in query {
//         if coords.x == 0 {
//             dbg!(coords);
//
//             dbg!(transform.translation);
//         }
//     }
// }

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

fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.7,

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

fn on_disocuppied(mut removed: RemovedComponents<Occupied>, query: Query<&Tile>) {
    for entity in removed.read() {
        if let Ok(tile) = query.get(entity) {
            // println!("\non_disocuppied: removed from tile: {:?}", tile);
        }
    }
}
