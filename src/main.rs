mod agent;
mod animation;
mod background;
mod brain;
mod constants;
mod consume;
mod events;
mod interaction;
mod interaction_queue;
mod message_animation;
mod pathfinder;
mod roof;
mod walk;
mod world;

use agent::{Agent, AgentPlugin};
use animation::AnimationPlugin;
use background::BackgroundPlugin;
use bevy::{color::palettes::css::*, log::LogPlugin, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TilePos};
use brain::*;
use constants::*;
use consume::*;
use interaction::InteractionPlugin;
use message_animation::MessageAnimationPlugin;
use roof::RoofPlugin;
use walk::{components::Walking, plugin::*};
use world::{components::*, grid::*, plugin::*, spatial_idx::*};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..Default::default()
                }),
        )
        .add_plugins(LdtkPlugin)
        .add_plugins(AgentPlugin)
        .add_plugins(RoofPlugin)
        .add_plugins(BrainPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(BackgroundPlugin)
        .add_plugins(WalkPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(ConsumePlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(MessageAnimationPlugin)
        .init_resource::<GizmoConfigStore>()
        .insert_resource(LevelSelection::index(0))
        .add_systems(PreStartup, setup_camera)
        .add_systems(
            Update,
            (
                mark_destination_on_map,
                draw_tile_gizmos,
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

fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,

            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(512.0, 512.0, 0.0),
    ));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("proj.ldtk").into(),

        ..Default::default()
    });
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

        let color = if tile_data.flags.contains(TileFlags::INSIDE) {
            if tile_data.flags.contains(TileFlags::FURNITURE) {
                Some(RED)
            } else {
                Some(GREEN)
            }
        } else if tile_data.flags.contains(TileFlags::WALL) {
            Some(GRAY)
        } else if tile_data.flags.contains(TileFlags::DOOR) {
            Some(YELLOW)
        } else if tile_data.flags.contains(TileFlags::FURNITURE) {
            Some(RED)
        } else {
            None
        };

        if let Some(color) = color {
            gizmos.rect_2d(
                pos.truncate(),
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
