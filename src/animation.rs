use bevy::{math::UVec2, prelude::*};

use crate::{events::FaceToFaceEvent, walk::components::Walking, world::components::GridPosition};

#[derive(Component, Clone, Copy)]
pub struct Animation {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AnimationDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Resource)]
pub struct CharacterAnimations {
    pub walk_up: Animation,
    pub walk_down: Animation,
    pub walk_left: Animation,
    pub walk_right: Animation,
}

#[derive(Resource)]
pub struct CharacterSpriteSheet {
    pub texture_atlas_layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_animations)
            .add_observer(turn_agents_to_face_each_other_system)
            .add_systems(Update, (animate_sprite_system, update_animation_direction));
    }
}

fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("body_dressed.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(64, 64), 9, 4, None, None);
    let texture_atlas_layout_handle = texture_atlas_layouts.add(layout);

    commands.insert_resource(CharacterSpriteSheet {
        texture_atlas_layout: texture_atlas_layout_handle,
        texture: texture_handle,
    });

    commands.insert_resource(CharacterAnimations {
        walk_down: Animation {
            first: 18,
            last: 26,
        },
        walk_up: Animation { first: 0, last: 8 },
        walk_left: Animation { first: 9, last: 17 },
        walk_right: Animation {
            first: 27,
            last: 35,
        },
    });
}

fn turn_agents_to_face_each_other_system(
    event: On<FaceToFaceEvent>,
    mut commands: Commands,
    agent_q: Query<&GridPosition>,
) {
    if let (Ok(pos_a), Ok(pos_b)) = (agent_q.get(event.agent_a), agent_q.get(event.agent_b)) {
        let dx = pos_a.x - pos_b.x;
        let dy = pos_a.y - pos_b.y;

        if dx > 0 {
            commands
                .entity(event.agent_a)
                .insert(AnimationDirection::Left);
            commands
                .entity(event.agent_b)
                .insert(AnimationDirection::Right);
        } else if dx < 0 {
            commands
                .entity(event.agent_a)
                .insert(AnimationDirection::Right);
            commands
                .entity(event.agent_b)
                .insert(AnimationDirection::Left);
        } else if dy > 0 {
            commands
                .entity(event.agent_a)
                .insert(AnimationDirection::Down);
            commands
                .entity(event.agent_b)
                .insert(AnimationDirection::Up);
        } else if dy < 0 {
            commands
                .entity(event.agent_a)
                .insert(AnimationDirection::Up);
            commands
                .entity(event.agent_b)
                .insert(AnimationDirection::Down);
        }
    }
}

fn update_animation_direction(
    mut query: Query<(&mut Animation, &AnimationDirection), Changed<AnimationDirection>>,
    animations: Res<CharacterAnimations>,
) {
    for (mut animation, direction) in query.iter_mut() {
        match direction {
            AnimationDirection::Up => {
                animation.first = animations.walk_up.first;
                animation.last = animations.walk_up.last;
            }
            AnimationDirection::Down => {
                animation.first = animations.walk_down.first;
                animation.last = animations.walk_down.last;
            }
            AnimationDirection::Left => {
                animation.first = animations.walk_left.first;
                animation.last = animations.walk_left.last;
            }
            AnimationDirection::Right => {
                animation.first = animations.walk_right.first;
                animation.last = animations.walk_right.last;
            }
        }
    }
}

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&Animation, &mut AnimationTimer, &mut Sprite), With<Walking>>,
) {
    for (animation, mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index == animation.last {
                    animation.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}
