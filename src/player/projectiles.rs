use bevy::prelude::*;

use crate::core_components::{
    CollisionCircle, Dead, Energy, HitPoints, Originator, Projectile, Shielded, Velocity,
};

use super::input::Action;
use super::Player;

pub(super) fn handle_shooting(
    mut commands: Commands,
    actions: Res<Input<Action>>,
    asset_server: Res<AssetServer>,
    mut players: Query<
        (
            Entity,
            &mut Energy,
            &Velocity,
            &Transform,
            Option<&Shielded>,
        ),
        With<Player>,
    >,
) {
    const PROJECTILE_ENERGY_COST: f32 = 1.0;
    const PROJECTILE_SPEED: f32 = 12.0;

    for (player, mut energy, velocity, transform, shielded) in players.iter_mut() {
        if actions.just_pressed(Action::Shoot(player))
            && energy.0 >= PROJECTILE_ENERGY_COST
            && shielded.is_none()
        {
            energy.0 -= PROJECTILE_ENERGY_COST;
            println!("Player {:?} energy: {}", player, energy.0);

            commands
                .spawn()
                .insert(Projectile)
                .insert(Originator(player))
                .insert(Velocity(
                    ((transform.rotation * Vec3::new(0.0, PROJECTILE_SPEED, 0.0)).truncate()
                        + velocity.0 / 2.0)
                        .normalize()
                        * PROJECTILE_SPEED,
                ))
                .insert_bundle(SpriteBundle {
                    texture: asset_server.load("images/projectile.png"),
                    transform: (*transform).with_scale(Vec3::splat(0.2))
                        * Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                    ..Default::default()
                })
                .insert(CollisionCircle { radius: 64.0 * 0.2 });
        }
    }
}

pub(super) fn handle_projectiles(
    mut commands: Commands,
    mut projectiles: Query<
        (
            Entity,
            &mut Transform,
            &Velocity,
            &Originator,
            &CollisionCircle,
        ),
        With<Projectile>,
    >,
    mut hp_entities: Query<
        (
            Entity,
            &mut HitPoints,
            &Transform,
            &CollisionCircle,
            Option<&Shielded>,
        ),
        Without<Projectile>,
    >,
) {
    for (_, mut transform, velocity, _, _) in projectiles.iter_mut() {
        transform.translation += velocity.0.extend(0.0);
    }

    for (projectile, transform, _, originator, collision) in projectiles.iter() {
        let hit = hp_entities
            .iter_mut()
            .filter(|(p, _, _, _, _)| *p != originator.0)
            .map(|(p, h, t, c, s)| {
                (
                    p,
                    h,
                    (transform.translation - t.translation).truncate().length()
                        - c.radius
                        - collision.radius,
                    s,
                )
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
            .filter(|a| a.2 <= 0.0)
            .map(|a| (a.0, a.1, a.3));

        if let Some((hp_entity, mut hp, shielded)) = hit {
            commands.entity(projectile).despawn();

            if shielded.is_none() && hp.0 > 0 {
                hp.0 -= 1;
                println!("Player {:?} hp: {}", hp_entity, hp.0);
                if hp.0 == 0 {
                    commands.entity(hp_entity).insert(Dead);
                }
            }
        }
    }
}
