use bevy::prelude::*;

pub fn custom_debug(entity: Entity, ctx: &str, msg: String) {
    info!("{} - {}: {}", entity, ctx, msg);
}
