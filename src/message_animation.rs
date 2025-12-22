use crate::background::MessageChannel;
use bevy::prelude::*;
use std::{collections::HashMap, time::Duration};

pub struct MessageAnimationPlugin;

impl Plugin for MessageAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MessageCollection>()
            .add_systems(Update, (handle_message_stream, show_message_animated));
    }
}

#[derive(Resource, Default)]
pub struct MessageCollection(pub HashMap<usize, (Entity, usize, Vec<char>, Timer)>); // entity, index, full text

fn handle_message_stream(
    mut commands: Commands,
    channel: Res<MessageChannel>,
    mut message_collection: ResMut<MessageCollection>,
    asset_server: Res<AssetServer>,
) {
    for msg in channel.receiver.try_iter() {
        if let Some((_, _, full_text, _)) = message_collection.0.get_mut(&msg.id) {
            for c in msg.content.chars() {
                full_text.push(c)
            }
        } else {
            let entity = commands
                .spawn((
                    Text::new(""),
                    TextFont {
                        // This font is loaded and will be used instead of the default font.
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 67.0,
                        ..default()
                    },
                    TextLayout::new_with_justify(Justify::Center),
                ))
                .id();

            message_collection.0.insert(
                msg.id,
                (
                    entity,
                    0,
                    msg.content.chars().collect(),
                    Timer::new(Duration::from_millis(50), TimerMode::Repeating),
                ),
            );
        }
    }
}

fn show_message_animated(
    mut message_collection: ResMut<MessageCollection>,
    mut query: Query<&mut Text>,
    time: Res<Time>,
) {
    for (_, (entity, idx, full_text, timer)) in &mut message_collection.0 {
        if timer.tick(time.delta()).just_finished() {
            if let Ok(mut text) = query.get_mut(*entity) {
                if let Some(next_char) = full_text.get(*idx) {
                    text.0.push(next_char.clone());
                    *idx += 1;
                }
            }
        }
    }
}
