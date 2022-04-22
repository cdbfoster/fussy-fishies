use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::animation::{Animation, AnimationStage};
use crate::background::spawn_bubble_group;
use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::core_components::{Dead, HitPoints};
use crate::player::PlayerConfiguration;
use crate::render::additional_pass::AdditionalPassPlugin;
use crate::State;

use self::animation::{bob, breathe, get_body_part, reset_animation, swim, AnimationState};
use self::camera::{setup_camera, BigFishCamera};
use self::model::{build_model, BodyPart};

mod animation;
mod camera;
mod model;

pub struct BigFishPlugin;

impl Plugin for BigFishPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationState::default())
            .insert_resource(AttentionTarget::default())
            .insert_resource(EatList::default())
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
                    .with_system(create_depth)
                    .with_system(add_dead_things_to_menu)
                    .with_system(update_attention_target)
                    .with_system(follow_attention_target)
                    .with_system(eat_dead_things.after("big_fish_animation")),
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

const ATTENTION_OFFSET: f32 = 200.0;

#[derive(Component)]
struct BigFish;

#[derive(Default)]
struct AttentionTarget(Vec3);

#[derive(Default)]
struct EatList(Vec<Entity>);

fn update_attention_target(
    mut attention_target: ResMut<AttentionTarget>,
    eat_list: Res<EatList>,
    player_configuration: Res<PlayerConfiguration>,
    hp_entities: Query<(&HitPoints, &Transform)>,
) {
    if let Some(eat_target) = eat_list.0.first() {
        let (_, transform) = hp_entities
            .get(*eat_target)
            .expect("could not find eat target");

        attention_target.0 = transform.translation;
        attention_target.0.z = -0.1;

        return;
    }

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

fn get_fish_space_scale() -> Vec3 {
    let min_dim = LOGICAL_WIDTH.min(LOGICAL_HEIGHT) as f32;
    Vec3::new(1.0 / min_dim, 1.0 / min_dim, 1.0 / -START_DEPTH)
}

fn follow_attention_target(
    mut animation_state: ResMut<AnimationState>,
    eat_list: Res<EatList>,
    attention_target: Res<AttentionTarget>,
    mut big_fish: Query<&mut Transform, With<BigFish>>,
) {
    let speed = if !eat_list.0.is_empty() { 0.05 } else { 0.0075 };

    let mut transform = big_fish.single_mut();

    let scale = get_fish_space_scale();

    let target = (attention_target.0 + Vec3::Y * ATTENTION_OFFSET) * scale;
    let pos = transform.translation * scale;

    let distance = target - pos;

    animation_state.swim_speed = 1.0 + distance.length() * 2000.0 * speed;
    transform.translation += distance * speed / scale;
}

fn create_depth(mut big_fish: Query<(&mut Sprite, &mut Transform), With<BigFish>>) {
    let (mut sprite, mut transform) = big_fish.single_mut();

    let closeness = 1.0 - transform.translation.z / START_DEPTH;

    sprite.color.set_a(closeness * 0.6);
    transform.scale = Vec3::splat(closeness * 0.4 + 0.6);
}

fn add_dead_things_to_menu(mut eat_list: ResMut<EatList>, dead_things: Query<Entity, Added<Dead>>) {
    eat_list.0.extend(dead_things.iter());
}

fn eat_dead_things(
    mut commands: Commands,
    mut animation_state: ResMut<AnimationState>,
    mut eat_list: ResMut<EatList>,
    asset_server: Res<AssetServer>,
    attention_target: Res<AttentionTarget>,
    time: Res<Time>,
    mut hideables: Query<&mut Visibility>,
    mut body_parts: Query<(&mut Transform, &BodyPart), Without<BigFish>>,
    big_fish: Query<&Transform, With<BigFish>>,
) {
    if let Some(eat_target) = eat_list.0.first() {
        let transform = big_fish.single();

        let scale = get_fish_space_scale();

        let target = (attention_target.0 + Vec3::Y * ATTENTION_OFFSET) * scale;
        let pos = transform.translation * scale;

        let distance = target - pos;

        if distance.length() < 0.5 {
            animation_state.chomping.tick(time.delta());
            let t = animation_state.chomping.percent();

            let bottom_jaw = Animation::new([
                AnimationStage::new(
                    0.0..0.9,
                    &|t| 1.0 - (1.0 - t) * (1.0 - t),
                    &|t, transform: &mut Transform| transform.translation.y -= 60.0 * t,
                ),
                AnimationStage::new(
                    0.9..1.0,
                    &|t| 1.0 - 2.0_f32.powf(10.0 * t - 10.0),
                    &|t, transform: &mut Transform| transform.translation.y -= 65.0 * t - 5.0,
                ),
            ]);

            let (mut transform, _) = get_body_part(&mut body_parts, BodyPart::BottomJaw);
            bottom_jaw.run(t, &mut transform);

            let top_jaw = Animation::new([
                AnimationStage::new(
                    0.0..0.9,
                    &|t| 1.0 - (1.0 - t) * (1.0 - t),
                    &|t, transform: &mut Transform| transform.translation.y += 90.0 * t,
                ),
                AnimationStage::new(
                    0.9..1.0,
                    &|t| 1.0 - 2.0_f32.powf(10.0 * t - 10.0),
                    &|t, transform: &mut Transform| transform.translation.y += 95.0 * t - 5.0,
                ),
            ]);

            let (mut transform, _) = get_body_part(&mut body_parts, BodyPart::TopJaw);
            top_jaw.run(t, &mut transform);

            let mut visibility = hideables.get_mut(*eat_target).expect("can't find hideable");

            if animation_state.chomping.just_finished() {
                visibility.is_visible = false;
                animation_state.chomping.reset();
                commands.entity(*eat_target).despawn_recursive();
                eat_list.0.remove(0);

                spawn_bubble_group(
                    &mut commands,
                    &asset_server,
                    attention_target.0,
                    10,
                    -50.0..50.0,
                    -50.0..50.0,
                    0.0..0.0001,
                );
            }
        }
    }
}
