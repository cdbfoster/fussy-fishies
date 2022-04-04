use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::query::WorldQuery;
use bevy::{ecs::query::FilterFetch, prelude::*};

use crate::animation::{Animation, AnimationStage};

use super::model::BodyPart;

#[derive(Component)]
pub(super) struct AnimationState {
    bobbing: Timer,
    breathing: Timer,
    swimming: Timer,
    pub(super) swim_speed: f32,
    pub(super) chomping: Timer,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            bobbing: Timer::from_seconds(4.0, true),
            breathing: Timer::from_seconds(3.0, true),
            swimming: Timer::from_seconds(5.0, true),
            swim_speed: 1.0,
            chomping: Timer::from_seconds(0.5, false),
        }
    }
}

pub(super) fn reset_animation(mut body_parts: Query<(&mut Transform, &BodyPart)>) {
    for (mut transform, body_part) in body_parts.iter_mut() {
        match body_part {
            BodyPart::Head => transform.translation = Vec3::new(0.0, -100.0, 0.0),
            BodyPart::TopJaw => transform.translation = Vec3::new(0.0, -30.0, 2.0),
            BodyPart::BottomJaw => transform.translation = Vec3::new(0.0, 30.0, 1.5),
            BodyPart::Lure => transform.rotation = Quat::IDENTITY,
            BodyPart::LeftFin | BodyPart::RightFin => transform.scale = Vec3::ONE,
        }
    }
}

pub(super) fn get_body_part<'a, F>(
    body_parts: &'a mut Query<(&mut Transform, &BodyPart), F>,
    part: BodyPart,
) -> (Mut<'a, Transform>, &'a BodyPart)
where
    F: WorldQuery,
    <F as WorldQuery>::Fetch: FilterFetch,
{
    body_parts
        .iter_mut()
        .find(|(_, b)| **b == part)
        .expect(&format!("could not find {:?}", part))
}

pub(super) fn breathe(
    mut animation_state: ResMut<AnimationState>,
    time: Res<Time>,
    mut body_parts: Query<(&mut Transform, &BodyPart)>,
) {
    animation_state.breathing.tick(time.delta());

    let mut animate_mouth =
        |part, amp| {
            let animation = Animation::new([AnimationStage::new(
                0.0..1.0,
                &|t| t,
                &|t, (amp, transform): (f32, &mut Transform)| {
                    transform.translation.y += amp + (2.0 * PI * t).sin() * amp
                },
            )]);

            let (mut transform, _) = get_body_part(&mut body_parts, part);
            animation.run(animation_state.breathing.percent(), (amp, &mut transform));
        };

    animate_mouth(BodyPart::BottomJaw, -5.0);
    animate_mouth(BodyPart::TopJaw, 2.5);
}

pub(super) fn bob(
    mut animation_state: ResMut<AnimationState>,
    time: Res<Time>,
    mut body_parts: Query<(&mut Transform, &BodyPart)>,
) {
    animation_state.bobbing.tick(time.delta());

    let animation = Animation::new([AnimationStage::new(
        0.0..1.0,
        &|t| t,
        &|t, transform: &mut Transform| {
            transform.translation.y += (2.0 * PI * t).sin() * 5.0;
        },
    )]);

    let (mut transform, _) = get_body_part(&mut body_parts, BodyPart::Head);
    animation.run(animation_state.bobbing.percent(), &mut transform);
}

pub(super) fn swim(
    mut animation_state: ResMut<AnimationState>,
    time: Res<Time>,
    mut body_parts: Query<(&mut Transform, &BodyPart)>,
) {
    let swim_speed = animation_state.swim_speed;
    animation_state
        .swimming
        .tick(Duration::from_secs_f32(time.delta_seconds() * swim_speed));

    let mut animate =
        |part, amp, phase| {
            let animation = Animation::new([AnimationStage::new(
                0.0..1.0,
                &|t| t,
                &|t, (amp, phase, transform): (f32, f32, &mut Transform)| {
                    transform.translation.x += (2.0 * PI * t + phase).sin() * amp
                },
            )]);

            let (mut transform, _) = get_body_part(&mut body_parts, part);
            animation.run(
                animation_state.swimming.percent(),
                (amp, phase, &mut transform),
            );
        };

    animate(BodyPart::Head, 10.0, 0.0);
    animate(BodyPart::TopJaw, 10.0, 0.5 * PI);
    animate(BodyPart::BottomJaw, 10.0, 0.5 * PI);

    let mut animate =
        |part, amp, phase| {
            let animation = Animation::new([AnimationStage::new(
                0.0..1.0,
                &|t| t,
                &|t, (amp, phase, transform): (f32, f32, &mut Transform)| {
                    transform.scale.x += (2.0 * PI * t + phase).sin() * amp * 0.5 - amp * 0.5
                },
            )]);

            let (mut transform, _) = get_body_part(&mut body_parts, part);
            animation.run(
                animation_state.swimming.percent(),
                (amp, phase, &mut transform),
            );
        };

    animate(BodyPart::LeftFin, 0.5, -0.05 * PI);
    animate(BodyPart::RightFin, 0.5, 0.95 * PI);
}
