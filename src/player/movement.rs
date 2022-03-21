use bevy::prelude::*;

use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::core_components::{AngularVelocity, CollisionCircle, Shielded, Velocity};

use super::input::Action;
use super::Player;

pub(super) const PLAYER_MAX_SPEED: f32 = 8.0;
pub(super) const PLAYER_MAX_ANGULAR_VELOCITY: f32 = 0.075;

pub(super) fn handle_movement(
    actions: Res<Input<Action>>,
    mut players: Query<
        (
            Entity,
            &mut Velocity,
            &mut AngularVelocity,
            &Transform,
            Option<&Shielded>,
        ),
        With<Player>,
    >,
) {
    const PLAYER_ACCELERATION: f32 = 0.3;
    const PLAYER_ANGULAR_ACCELERATION: f32 = 0.015;

    for (player, mut velocity, mut angular_velocity, transform, shielded) in players.iter_mut() {
        if actions.pressed(Action::MoveForward(player)) && shielded.is_none() {
            velocity.0 += (transform.rotation * Vec3::new(0.0, PLAYER_ACCELERATION, 0.0)).truncate()
        }

        if actions.pressed(Action::TurnLeft(player)) {
            angular_velocity.0 += PLAYER_ANGULAR_ACCELERATION;
        }

        if actions.pressed(Action::TurnRight(player)) {
            angular_velocity.0 -= PLAYER_ANGULAR_ACCELERATION;
        }
    }
}

pub(super) fn move_players(
    mut players: Query<
        (
            &mut Velocity,
            &mut AngularVelocity,
            &mut Transform,
            Option<&Shielded>,
        ),
        With<Player>,
    >,
) {
    const PLAYER_DECELERATION: f32 = 0.02;
    const PLAYER_ANGULAR_DECELERATION: f32 = 0.2;
    const PLAYER_SHIELD_BRAKE: f32 = 0.1;

    for (mut velocity, mut angular_velocity, mut transform, shielded) in players.iter_mut() {
        if velocity.0.length() > PLAYER_MAX_SPEED {
            velocity.0 = velocity.0.normalize() * PLAYER_MAX_SPEED;
        }

        angular_velocity.0 = angular_velocity
            .0
            .min(PLAYER_MAX_ANGULAR_VELOCITY)
            .max(-PLAYER_MAX_ANGULAR_VELOCITY);

        transform.translation += velocity.0.extend(0.0);
        transform.rotation *= Quat::from_rotation_z(angular_velocity.0);

        velocity.0 *= 1.0 - PLAYER_DECELERATION;
        if shielded.is_some() {
            velocity.0 *= 1.0 - PLAYER_SHIELD_BRAKE;
        }

        angular_velocity.0 *= 1.0 - PLAYER_ANGULAR_DECELERATION;
    }
}

// XXX When https://github.com/bevyengine/bevy/issues/3651 gets fixed, change this to a With<Player> query.
pub(super) fn handle_collision(
    mut players: Query<(&Player, &mut Velocity, &mut Transform, &CollisionCircle)>,
) {
    const COLLISION_ITERATIONS: usize = 10;
    const COLLISION_MARGIN: f32 = 0.1;

    let mut found_collision = false;

    'check_all: for _ in 0..COLLISION_ITERATIONS {
        // Collide players against others
        let mut combinations = players.iter_combinations_mut();
        while let Some(
            [(_, _, mut transform_a, collision_a), (_, _, mut transform_b, collision_b)],
        ) = combinations.fetch_next()
        {
            let vector_between = transform_b.translation - transform_a.translation;
            let distance = vector_between.length();

            if distance < collision_a.radius + collision_b.radius {
                found_collision = true;
            } else {
                continue;
            }

            let correction =
                (distance - collision_a.radius - collision_b.radius) / 2.0 - COLLISION_MARGIN;
            let collision_normal = vector_between.normalize_or_zero();

            transform_a.translation += collision_normal * correction;
            transform_b.translation -= collision_normal * correction;
        }

        // Collide players against walls
        for (_, mut velocity, mut transform, collision) in players.iter_mut() {
            if transform.translation.x + collision.radius > LOGICAL_WIDTH as f32 {
                transform.translation.x =
                    LOGICAL_WIDTH as f32 - collision.radius - COLLISION_MARGIN;
                velocity.0 = Vec2::new(0.0, Vec2::new(0.0, 1.0).dot(velocity.0));
                found_collision = true;
            }

            if transform.translation.y + collision.radius > LOGICAL_HEIGHT as f32 {
                transform.translation.y =
                    LOGICAL_HEIGHT as f32 - collision.radius - COLLISION_MARGIN;
                velocity.0 = Vec2::new(Vec2::new(1.0, 0.0).dot(velocity.0), 0.0);
                found_collision = true;
            }

            if transform.translation.x - collision.radius < 0.0 {
                transform.translation.x = collision.radius + COLLISION_MARGIN;
                velocity.0 = Vec2::new(0.0, Vec2::new(0.0, 1.0).dot(velocity.0));
                found_collision = true;
            }

            if transform.translation.y - collision.radius < 0.0 {
                transform.translation.y = collision.radius - COLLISION_MARGIN;
                velocity.0 = Vec2::new(Vec2::new(1.0, 0.0).dot(velocity.0), 0.0);
                found_collision = true;
            }
        }

        if !found_collision {
            break 'check_all;
        }
    }
}
