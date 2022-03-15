use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;

use crate::animation::{Animation, AnimationStage};
use crate::core_components::{AngularVelocity, Dead, Velocity};
use crate::energy_orbs::{EnergyOrb, RespawnTimer as EnergyOrbRespawnTimer};

use super::movement::{PLAYER_MAX_ANGULAR_VELOCITY, PLAYER_MAX_SPEED};
use super::{BodyPart, Player};

#[derive(Component)]
pub(super) struct SwimmingAnimation(pub(super) Timer);

pub(super) fn animate_swimming(
    time: Res<Time>,
    mut players: Query<
        (
            &mut SwimmingAnimation,
            &Velocity,
            &AngularVelocity,
            &Children,
        ),
        (With<Player>, Without<BodyPart>),
    >,
    mut body_parts: Query<(&mut Transform, &BodyPart, Option<&Children>)>,
) {
    for (mut timer, velocity, angular_velocity, children) in players.iter_mut() {
        let animation_strength = (velocity.0.length() / PLAYER_MAX_SPEED
            + (angular_velocity.0.abs() / PLAYER_MAX_ANGULAR_VELOCITY).powi(3))
        .max(0.3)
        .min(1.0);

        // Speed up the animation based on how fast we're moving.
        timer.0.tick(Duration::from_secs_f32(
            2.06 * (animation_strength - 0.15) * time.delta_seconds(),
        ));

        let mut body_animation = |part, pos_strength, rot_strength, offset| {
            let animation = Animation::new([AnimationStage::new(
                0.0..1.0,
                &|t| t,
                &|t, (transform, sp, sr, o): (&mut Transform, f32, f32, f32)| {
                    transform.translation.x = sp * (2.0 * PI * t - o).sin();
                    transform.rotation = Quat::from_rotation_z(-sr * (2.0 * PI * t - o).cos());
                },
            )]);

            let entity = children
                .iter()
                .find(|c| {
                    body_parts
                        .get_component::<BodyPart>(**c)
                        .ok()
                        .filter(|b| **b == part)
                        .is_some()
                })
                .cloned()
                .unwrap();

            let mut transform = body_parts
                .get_component_mut::<Transform>(entity)
                .expect("cannot find entity");

            animation.run(
                timer.0.percent(),
                (&mut transform, pos_strength, rot_strength, offset),
            );

            entity
        };

        let head_entity = body_animation(
            BodyPart::Head,
            12.0 * animation_strength,
            animation_strength * PI / 16.0,
            0.0,
        );

        body_animation(BodyPart::Body, 15.0 * animation_strength, 0.0, PI / 4.0);

        body_animation(
            BodyPart::Tail,
            18.0 * animation_strength,
            animation_strength * PI / 4.0,
            PI / 2.0,
        );

        let mut fin_animation = |part, strength, side| {
            let animation = Animation::new([AnimationStage::new(
                0.0..1.0,
                &|t| t,
                &|t, (transform, strength, side, av): (&mut Transform, f32, f32, f32)| {
                    transform.rotation = Quat::from_rotation_z(side * 0.15 - 3.0 * av).slerp(
                        Quat::from_rotation_z(side * 0.60 - 12.0 * av),
                        ((2.0 * PI * t + PI * 0.75).sin() * strength + 1.0) / 2.0,
                    );
                },
            )]);

            let entity = body_parts
                .get_component::<Children>(head_entity)
                .expect("cannot find entity")
                .iter()
                .find(|c| {
                    body_parts
                        .get_component::<BodyPart>(**c)
                        .ok()
                        .filter(|b| **b == part)
                        .is_some()
                })
                .cloned()
                .unwrap();

            let mut transform = body_parts
                .get_component_mut::<Transform>(entity)
                .expect("cannot find entity");

            animation.run(
                timer.0.percent(),
                (&mut transform, strength, side, angular_velocity.0),
            );
        };

        fin_animation(BodyPart::RightFin, animation_strength, 1.0);
        fin_animation(BodyPart::LeftFin, -animation_strength, -1.0);
    }
}

pub(super) fn animate_eyes(
    mut body_parts: Query<(
        &mut Transform,
        &GlobalTransform,
        &BodyPart,
        Option<&Children>,
    )>,
    players: Query<
        (Entity, &Transform, &Children),
        (With<Player>, Without<BodyPart>, Without<Dead>),
    >,
    orbs: Query<&EnergyOrb, Without<EnergyOrbRespawnTimer>>,
) {
    const FIELD_OF_VIEW: f32 = PI / 4.0;
    const LOOK_SPEED: f32 = 0.2;

    for (entity, transform, children) in players.iter() {
        let look = (transform.rotation * Vec3::new(0.0, 1.0, 0.0)).truncate();

        let target = players
            .iter()
            .filter(|(e, _, _)| *e != entity)
            .map(|(_, t, _)| t.translation.truncate())
            .chain(orbs.iter().map(|o| o.0))
            .map(|p| {
                let view_vector = p - transform.translation.truncate();
                (
                    look.angle_between(view_vector).abs(),
                    view_vector.length(),
                    p,
                )
            })
            .filter(|(a, _, _)| *a <= FIELD_OF_VIEW)
            .min_by(|(_, d, _), (_, e, _)| d.partial_cmp(e).unwrap())
            .map(|(_, _, p)| p);

        let head_entity = children
            .iter()
            .find(|c| {
                body_parts
                    .get_component::<BodyPart>(**c)
                    .ok()
                    .filter(|b| **b == BodyPart::Head)
                    .is_some()
            })
            .cloned()
            .unwrap();

        let head_children = body_parts
            .get_component::<Children>(head_entity)
            .expect("cannot find head entity")
            .clone();

        let mut look_at = |pos: Option<Vec2>, part| {
            let entity = head_children
                .iter()
                .find(|c| {
                    body_parts
                        .get_component::<BodyPart>(**c)
                        .ok()
                        .filter(|b| **b == part)
                        .is_some()
                })
                .cloned()
                .unwrap();

            let (mut t, gt, _, _) = body_parts.get_mut(entity).expect("cannot find entity");

            let target_rotation = if let Some(pos) = pos {
                let vector = pos - gt.translation.truncate();
                Quat::from_rotation_z(look.angle_between(vector))
            } else {
                Quat::from_rotation_z(0.0)
            };

            t.rotation = t.rotation.slerp(target_rotation, LOOK_SPEED);
        };

        look_at(target, BodyPart::RightEye);
        look_at(target, BodyPart::LeftEye);
    }
}
