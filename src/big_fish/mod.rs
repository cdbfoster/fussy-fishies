use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::core_components::HitPoints;
use crate::player::PlayerConfiguration;
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
            .insert_resource(AttentionTarget::default())
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
                    .with_system(reset_animation.before("big_fish_animation"))
                    .with_system(update_attention_target)
                    .with_system(follow_attention_target)
                    .with_system(create_depth),
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

const FOLLOW_FACTOR: f32 = 0.0075;
const START_DEPTH: f32 = -20.0;
const START_SCALE: f32 = 1.0;

#[derive(Component)]
struct BigFish;

#[derive(Default)]
struct AttentionTarget(Vec3);

fn update_attention_target(
    mut attention_target: ResMut<AttentionTarget>,
    player_configuration: Res<PlayerConfiguration>,
    hp_entities: Query<(&HitPoints, &Transform)>,
) {
    let minimum_hp = hp_entities.iter().min_by_key(|(hp, _)| hp.0);

    if let Some((&HitPoints(minimum_hp), _)) = minimum_hp {
        let starting_maximum_hp = player_configuration
            .0
            .iter()
            .filter_map(|p| p.clone().map(|c| c.hp.0))
            .max()
            .unwrap();
        let mut count = 0;
        let mut sum = Vec2::ZERO;

        for (_, transform) in hp_entities.iter().filter(|(hp, _)| hp.0 == minimum_hp) {
            count += 1;
            sum += transform.translation.truncate();
        }

        attention_target.0 = (sum / count as f32)
            .extend(START_DEPTH * minimum_hp as f32 / starting_maximum_hp as f32);
    } else {
        attention_target.0 = Vec3::new(LOGICAL_WIDTH as f32, LOGICAL_HEIGHT as f32, START_DEPTH);
    }
}

fn follow_attention_target(
    mut animation_state: ResMut<AnimationState>,
    attention_target: Res<AttentionTarget>,
    mut big_fish: Query<&mut Transform, With<BigFish>>,
) {
    let mut transform = big_fish.single_mut();

    let min_dim = LOGICAL_WIDTH.min(LOGICAL_HEIGHT) as f32;
    let scale = Vec3::new(1.0 / min_dim, 1.0 / min_dim, 1.0 / -START_DEPTH);

    let target = attention_target.0 * scale;
    let pos = transform.translation * scale;

    let distance = target - pos;

    animation_state.swim_speed = 1.0 + distance.length() * 15.0;
    transform.translation += distance * FOLLOW_FACTOR / scale;
}

fn create_depth(mut big_fish: Query<(&mut Sprite, &mut Transform), With<BigFish>>) {
    let (mut sprite, mut transform) = big_fish.single_mut();

    let closeness = 1.0 - transform.translation.z / START_DEPTH;

    sprite.color.set_a(closeness * 0.6);
    transform.scale = Vec3::splat(closeness * 0.4 + 0.6);
}
