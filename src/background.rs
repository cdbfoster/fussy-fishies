use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::State;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .add_system_set(SystemSet::on_enter(State::Game).with_system(create_background));
    }
}

fn create_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/background.png"),
            transform: Transform::from_translation(Vec3::new(
                LOGICAL_WIDTH as f32 / 2.0,
                LOGICAL_HEIGHT as f32 / 2.0,
                -1.0,
            )),
            ..Default::default()
        })
        .insert(RenderLayers::layer(1));
}
