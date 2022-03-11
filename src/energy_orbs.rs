use std::time::Duration;

use bevy::prelude::*;

use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::core_components::{CollisionCircle, Energy};
use crate::player::{Player, PLAYER_SCALE};
use crate::State;

pub struct EnergyOrbsPlugin;

impl Plugin for EnergyOrbsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(State::Game).with_system(spawn_starting_orbs))
            .add_system_set(
                SystemSet::on_update(State::Game)
                    .with_system(player_pickup.label("player_pickup"))
                    .with_system(respawn_orbs)
                    .with_system(change_player_size.after("player_pickup")),
            );
    }
}

#[derive(Clone, Component)]
struct EnergyOrb(Vec2);

#[derive(Component)]
struct RespawnTimer(Timer);

const ORB_POSITIONS: [(f32, f32); 8] = [
    (LOGICAL_WIDTH * 0.45, LOGICAL_HEIGHT * 0.575),
    (LOGICAL_WIDTH * 0.45, LOGICAL_HEIGHT * 0.425),
    (LOGICAL_WIDTH * 0.55, LOGICAL_HEIGHT * 0.575),
    (LOGICAL_WIDTH * 0.55, LOGICAL_HEIGHT * 0.425),
    (LOGICAL_WIDTH * 0.10, LOGICAL_HEIGHT * 0.90),
    (LOGICAL_WIDTH * 0.10, LOGICAL_HEIGHT * 0.10),
    (LOGICAL_WIDTH * 0.90, LOGICAL_HEIGHT * 0.90),
    (LOGICAL_WIDTH * 0.90, LOGICAL_HEIGHT * 0.10),
];

const ORB_SCALE: f32 = 0.3;

fn spawn_starting_orbs(mut commands: Commands, asset_server: Res<AssetServer>) {
    for (x, y) in ORB_POSITIONS.iter().copied() {
        commands
            .spawn()
            .insert(EnergyOrb(Vec2::new(x, y)))
            .insert_bundle(SpriteBundle {
                texture: asset_server.load("images/orb.png"),
                transform: Transform::from_scale(Vec3::splat(ORB_SCALE))
                    .with_translation(Vec3::new(x, y, 0.0)),
                ..Default::default()
            })
            .insert(CollisionCircle {
                radius: 64.0 * ORB_SCALE,
            });
    }
}

fn player_pickup(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Energy, &Transform, &CollisionCircle), With<Player>>,
    mut orbs: Query<(Entity, &mut Visibility, &EnergyOrb, &CollisionCircle), Without<RespawnTimer>>,
) {
    const MAX_ENERGY: f32 = 20.0;
    const ORB_ENERGY_BOOST: f32 = 3.0;
    const ORB_RESPAWN_SECS: f32 = 15.0;

    for (orb_entity, mut orb_visibility, orb, orb_collision) in orbs.iter_mut() {
        let collision = players
            .iter_mut()
            .map(|(p, e, t, c)| {
                (
                    p,
                    e,
                    (orb.0 - t.translation.truncate()).length() - c.radius - orb_collision.radius,
                )
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
            .filter(|a| a.2 <= 0.0)
            .filter(|a| a.1 .0 < MAX_ENERGY);

        if let Some((player_entity, mut player_energy, _)) = collision {
            orb_visibility.is_visible = false;
            commands
                .entity(orb_entity)
                .insert(RespawnTimer(Timer::from_seconds(ORB_RESPAWN_SECS, false)));
            player_energy.0 = (player_energy.0 + ORB_ENERGY_BOOST).min(MAX_ENERGY);
            println!("Player {:?} energy: {}", player_entity, player_energy.0);
        }
    }
}

fn respawn_orbs(
    mut commands: Commands,
    time: Res<Time>,
    players: Query<(&Transform, &CollisionCircle), With<Player>>,
    mut orbs: Query<(
        Entity,
        &mut Visibility,
        &mut RespawnTimer,
        &EnergyOrb,
        &CollisionCircle,
    )>,
) {
    for (orb_entity, mut orb_visibility, mut respawn_timer, orb, orb_collision) in orbs.iter_mut() {
        respawn_timer
            .0
            .tick(Duration::from_secs_f32(time.delta_seconds()));

        if respawn_timer.0.finished()
            && !players
                .iter()
                .map(|(t, c)| {
                    (orb.0 - t.translation.truncate()).length() - c.radius - orb_collision.radius
                })
                .any(|d| d <= 0.0)
        {
            orb_visibility.is_visible = true;
            commands.entity(orb_entity).remove::<RespawnTimer>();
        }
    }
}

fn change_player_size(
    //mut commands: Commands,
    mut players: Query<(&mut Transform, &mut CollisionCircle, &Energy), With<Player>>,
) {
    const ENERGY_SCALE_MULTIPLIER: f32 = 1.06;

    for (mut transform, mut collision_circle, energy) in players.iter_mut() {
        let target_scale_factor = PLAYER_SCALE * ENERGY_SCALE_MULTIPLIER.powf(energy.0.max(1.0));
        let next_scale_factor =
            transform.scale.z + (target_scale_factor - transform.scale.z) * 0.05;

        let base_scale = transform.scale / transform.scale.z;
        transform.scale = base_scale * next_scale_factor;

        collision_circle.radius = 128.0 * next_scale_factor;
    }
}
