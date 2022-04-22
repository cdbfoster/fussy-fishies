use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;

use crate::background::spawn_bubble_group;
use crate::core_components::{
    CollisionCircle, Dead, Energy, HitPoints, Originator, Projectile, Shielded, Velocity,
};

use super::input::Action;
use super::model::BodyPart;
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
                    ..default()
                })
                .insert(CollisionCircle { radius: 64.0 * 0.2 });
        }
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub(super) struct HpEntityQuery<'w> {
    entity: Entity,
    hp: &'w mut HitPoints,
    transform: &'w Transform,
    collision: &'w CollisionCircle,
    shielded: Option<&'w Shielded>,
    children: Option<&'w Children>,
}

pub(super) fn handle_projectiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
    mut hp_entities: Query<HpEntityQuery, Without<Projectile>>,
    body_parts: Query<(Entity, &GlobalTransform), With<BodyPart>>,
) {
    for (_, mut transform, velocity, _, _) in projectiles.iter_mut() {
        transform.translation += velocity.0.extend(0.0);
    }

    for (projectile, mut transform, _, originator, collision) in projectiles.iter_mut() {
        let hit = hp_entities
            .iter_mut()
            .filter(|e| e.entity != originator.0)
            .map(|e| {
                let distance = (transform.translation - e.transform.translation)
                    .truncate()
                    .length()
                    - e.collision.radius
                    - collision.radius;

                (e, distance)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .filter(|a| a.1 <= 0.0)
            .map(|(e, _)| e);

        if let Some(mut e) = hit {
            if e.shielded.is_none() {
                commands.entity(projectile).remove::<Projectile>();

                let (parent_entity, parent_transform) = if let Some(children) = e.children {
                    children
                        .iter()
                        .filter_map(|c| body_parts.get(*c).ok())
                        .map(|(c, t)| (c, t, (t.translation - transform.translation).length()))
                        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                        .map(|(c, t, _)| (c, (*t).into()))
                        .unwrap_or((e.entity, e.transform.clone()))
                } else {
                    (e.entity, e.transform.clone())
                };

                let mut vector = transform.translation - parent_transform.translation;
                vector = parent_transform.rotation.inverse() * vector;
                vector *= 0.8 / parent_transform.scale;
                vector.z += 3.0;

                *transform = Transform::from_translation(vector)
                    .with_scale(transform.scale / parent_transform.scale);

                commands.entity(parent_entity).push_children(&[projectile]);

                if e.hp.0 > 0 {
                    e.hp.0 -= 1;
                    println!("Player {:?} hp: {}", e.entity, e.hp.0);
                    if e.hp.0 == 0 {
                        commands.entity(e.entity).insert(Dead);
                    }
                }
            } else {
                commands.entity(projectile).despawn();
                spawn_bubble_group(
                    &mut commands,
                    &asset_server,
                    transform.translation,
                    3,
                    -10.0..10.0,
                    -10.0..10.0,
                    0.0..0.001,
                );
            }
        }
    }
}
