use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::render::additional_pass::AdditionalPassPlugin;
use crate::State;

use self::animation::{bob, breathe, reset_animation, swim, AnimationState};
use self::camera::{setup_camera, BigFishCamera};
use self::model::build_model;

mod animation;
mod camera;
mod model;

pub struct BigFishPlugin;

impl Plugin for BigFishPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationState::default())
            .add_plugin(AdditionalPassPlugin::<BigFishCamera>::new(
                "big_fish_pass",
                Some("foreground_pass"),
            ))
            .add_system_set(
                SystemSet::on_enter(State::Game)
                    .with_system(setup_camera)
                    .with_system(build_model),
            )
            .add_system_set(
                SystemSet::on_update(State::Game)
                    .with_system(reset_animation.before("big_fish_animation")),
            )
            .add_system_set(
                SystemSet::on_update(State::Game)
                    .label("big_fish_animation")
                    .with_system(bob)
                    .with_system(breathe)
                    .with_system(swim),
            );
    }
}

const BIG_FISH_LAYER: RenderLayers = RenderLayers::layer(2);

const START_DEPTH: f32 = -20.0;
const START_SCALE: f32 = 1.0;

#[derive(Component)]
struct BigFish;
