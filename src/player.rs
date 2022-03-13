#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;

use crate::animation::{Animation, AnimationStage};
use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::core_components::{
    AngularVelocity, CollisionCircle, Dead, Energy, HitPoints, Lives, Originator, Projectile,
    Shield, Shielded, Velocity,
};
use crate::energy_orbs::{EnergyOrb, RespawnTimer as EnergyOrbRespawnTimer};
use crate::State;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerConfiguration(vec![None; 4]))
            .add_event::<PlayerInput>()
            .add_system_set(SystemSet::on_enter(State::Game).with_system(create_players))
            .add_system_set(
                SystemSet::on_update(State::Game)
                    .with_system(gather_player_input.label("gather_input"))
                    .with_system(
                        deshield_players
                            .after("gather_input")
                            .before("handle_input"),
                    )
                    .with_system(
                        handle_player_input
                            .label("handle_input")
                            .after("gather_input"),
                    )
                    .with_system(
                        enforce_speed_limits
                            .after("handle_input")
                            .before("move_players"),
                    )
                    .with_system(move_players.label("move_players"))
                    .with_system(
                        handle_collision
                            .label("handle_collision")
                            .after("move_players"),
                    )
                    .with_system(move_projectiles.label("move_projectiles"))
                    .with_system(
                        detect_projectile_hits
                            .label("detect_projectile_hits")
                            .after("move_projectiles"),
                    )
                    .with_system(animate_swimming.after("move_players"))
                    .with_system(animate_eyes)
                    .with_system(dying),
            );
    }
}

fn dying(mut commands: Commands, query: Query<Entity, Added<Dead>>) {
    for dead in query.iter() {
        commands.entity(dead).despawn_recursive();
    }
}

#[derive(Clone, Component)]
pub struct Player;

pub struct PlayerConfiguration(pub Vec<Option<PlayerConfigurationBundle>>);

#[derive(Bundle, Clone)]
pub struct PlayerConfigurationBundle {
    pub keymap: KeyMap,
    pub color: PlayerColor,
    pub hp: HitPoints,
    pub lives: Lives,
}

#[derive(Clone, Component, Copy)]
pub struct KeyMap {
    pub forward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub shoot: KeyCode,
}

#[derive(Clone, Component)]
pub struct PlayerColor(pub Color);

#[derive(Bundle, Default)]
struct PlayerObjectBundle {
    velocity: Velocity,
    angular_velocity: AngularVelocity,
    energy: Energy,
}

#[derive(Clone, Component, Copy, PartialEq)]
enum BodyPart {
    Head,
    Body,
    Tail,
    RightFin,
    LeftFin,
    RightEye,
    LeftEye,
}

enum PlayerInput {
    Move(Entity),
    Left(Entity),
    Right(Entity),
    Shield(Entity),
    Shoot(Entity),
}

impl PlayerInput {
    fn player(&self) -> Entity {
        match self {
            PlayerInput::Move(e)
            | PlayerInput::Left(e)
            | PlayerInput::Right(e)
            | PlayerInput::Shield(e)
            | PlayerInput::Shoot(e) => *e,
        }
    }
}

pub const PLAYER_SCALE: f32 = 0.4;
pub const PLAYER_SHIELD_SCALE: f32 = 1.2;

fn create_players(
    mut commands: Commands,
    player_config: Res<PlayerConfiguration>,
    asset_server: Res<AssetServer>,
) {
    const PLAYER_START_POSITIONS: [(f32, f32); 4] = [
        (LOGICAL_WIDTH * 0.35, LOGICAL_HEIGHT * 0.70),
        (LOGICAL_WIDTH * 0.65, LOGICAL_HEIGHT * 0.70),
        (LOGICAL_WIDTH * 0.35, LOGICAL_HEIGHT * 0.30),
        (LOGICAL_WIDTH * 0.65, LOGICAL_HEIGHT * 0.30),
    ];

    const PLAYER_START_ANGLES: [f32; 4] = [PI / 4.0, -PI / 4.0, 3.0 * PI / 4.0, -3.0 * PI / 4.0];

    for (i, player) in player_config.0.iter().cloned().flatten().enumerate() {
        commands
            .spawn()
            .insert(Player)
            .insert_bundle(SpriteBundle {
                texture: asset_server.load("images/player/root.png"),
                transform: Transform::from_scale(Vec3::splat(PLAYER_SCALE))
                    .with_translation(Vec3::new(
                        PLAYER_START_POSITIONS[i].0,
                        PLAYER_START_POSITIONS[i].1,
                        1.0,
                    ))
                    .with_rotation(Quat::from_rotation_z(PLAYER_START_ANGLES[i])),
                ..Default::default()
            })
            .with_children(|root| {
                root.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("images/player/head.png"),
                    transform: Transform::from_translation(Vec3::new(0.0, 20.0, 1.0)),
                    sprite: Sprite {
                        color: player.color.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BodyPart::Head)
                .with_children(|head| {
                    head.spawn_bundle(SpriteBundle {
                        texture: asset_server.load("images/player/fin.png"),
                        transform: Transform::from_translation(Vec3::new(115.0, 0.0, 1.0))
                            .with_rotation(Quat::from_rotation_z(0.375)),
                        sprite: Sprite {
                            color: player.color.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(BodyPart::RightFin);

                    head.spawn_bundle(SpriteBundle {
                        texture: asset_server.load("images/player/fin.png"),
                        transform: Transform::from_translation(Vec3::new(-115.0, 0.0, 1.0))
                            .with_rotation(Quat::from_rotation_z(-0.375)),
                        sprite: Sprite {
                            color: player.color.0,
                            flip_x: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(BodyPart::LeftFin);

                    head.spawn_bundle(SpriteBundle {
                        texture: asset_server.load("images/player/eye-open.png"),
                        transform: Transform::from_translation(Vec3::new(85.0, 62.0, 1.0)),
                        sprite: Sprite {
                            color: player.color.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(BodyPart::RightEye);

                    head.spawn_bundle(SpriteBundle {
                        texture: asset_server.load("images/player/eye-open.png"),
                        transform: Transform::from_translation(Vec3::new(-85.0, 62.0, 1.0)),
                        sprite: Sprite {
                            color: player.color.0,
                            flip_x: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(BodyPart::LeftEye);

                    head.spawn_bundle(SpriteBundle {
                        texture: asset_server.load("images/player/mouth.png"),
                        transform: Transform::from_translation(Vec3::new(0.0, 100.0, 1.0)),
                        sprite: Sprite {
                            color: player.color.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });

                root.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("images/player/body.png"),
                    transform: Transform::from_translation(Vec3::new(0.0, -60.0, 2.0)),
                    sprite: Sprite {
                        color: player.color.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BodyPart::Body);

                root.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("images/player/tail.png"),
                    transform: Transform::from_translation(Vec3::new(0.0, -105.0, 3.0)),
                    sprite: Sprite {
                        color: player.color.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BodyPart::Tail);
            })
            .insert_bundle(player)
            .insert_bundle(PlayerObjectBundle::default())
            .insert(CollisionCircle {
                radius: 128.0 * PLAYER_SCALE,
            })
            .insert(SwimmingAnimation(Timer::from_seconds(0.333, true)));
    }
}

fn gather_player_input(
    mut player_input: EventWriter<PlayerInput>,
    keyboard: Res<Input<KeyCode>>,
    query: Query<(Entity, &KeyMap), With<Player>>,
) {
    for (player, keymap) in query.iter() {
        if keyboard.pressed(keymap.forward) {
            player_input.send(PlayerInput::Move(player));
        }

        if keyboard.just_pressed(keymap.shoot) {
            player_input.send(PlayerInput::Shoot(player));
        }

        if keyboard.pressed(keymap.left) && keyboard.pressed(keymap.right) {
            player_input.send(PlayerInput::Shield(player));
        } else if keyboard.pressed(keymap.left) {
            player_input.send(PlayerInput::Left(player));
        } else if keyboard.pressed(keymap.right) {
            player_input.send(PlayerInput::Right(player));
        }
    }
}

fn deshield_players(
    mut commands: Commands,
    mut player_input: EventReader<PlayerInput>,
    players: Query<(Entity, Option<&Shielded>, &Children), With<Player>>,
    shields: Query<Entity, With<Shield>>,
) {
    let mut shielded_players = players
        .iter()
        .filter_map(|(e, s, c)| s.and(Some((e, c))))
        .collect::<Vec<_>>();

    for input in player_input.iter() {
        if let PlayerInput::Shield(player) = input {
            if let Some(found) = shielded_players.iter().position(|p| p.0 == *player) {
                shielded_players.remove(found);
            }
        }
    }

    for (player, children) in shielded_players {
        commands.entity(player).remove::<Shielded>();

        let shield = children
            .iter()
            .copied()
            .find(|c| shields.get(*c).is_ok())
            .expect("cannot find shield entity");

        commands.entity(shield).despawn_recursive();
    }
}

fn handle_player_input(
    mut commands: Commands,
    time: Res<Time>,
    mut player_input: EventReader<PlayerInput>,
    mut players: Query<
        (
            Entity,
            &mut Velocity,
            &mut AngularVelocity,
            &mut Energy,
            &Transform,
            Option<&Shielded>,
            &Children,
        ),
        With<Player>,
    >,
    shields: Query<Entity, With<Shield>>,
    asset_server: Res<AssetServer>,
) {
    const PLAYER_ACCELERATION: f32 = 0.3;
    const PLAYER_ANGULAR_ACCELERATION: f32 = 0.015;

    for input in player_input.iter() {
        let player = input.player();
        let (entity, mut velocity, mut angular_velocity, mut energy, transform, shielded, children) =
            players.get_mut(player).expect("cannot find player entity");

        match input {
            PlayerInput::Move(_) => {
                if shielded.is_none() {
                    velocity.0 +=
                        (transform.rotation * Vec3::new(0.0, PLAYER_ACCELERATION, 0.0)).truncate()
                }
            }
            PlayerInput::Left(_) => {
                angular_velocity.0 += PLAYER_ANGULAR_ACCELERATION;
            }
            PlayerInput::Right(_) => {
                angular_velocity.0 -= PLAYER_ANGULAR_ACCELERATION;
            }
            PlayerInput::Shield(_) => handle_player_shielding(
                &mut commands,
                &time,
                entity,
                &mut energy.0,
                shielded.is_some(),
                children,
                &shields,
                &asset_server,
            ),
            PlayerInput::Shoot(_) => handle_player_shooting(
                &mut commands,
                entity,
                &mut energy.0,
                &velocity.0,
                transform,
                shielded.is_some(),
                &asset_server,
            ),
        }
    }
}

fn handle_player_shielding(
    commands: &mut Commands,
    time: &Res<Time>,
    entity: Entity,
    energy: &mut f32,
    shielded: bool,
    children: &Children,
    shields: &Query<Entity, With<Shield>>,
    asset_server: &Res<AssetServer>,
) {
    const SHIELD_DRAIN_RATE: f32 = 2.0;

    if shielded {
        *energy -= time.delta_seconds() * SHIELD_DRAIN_RATE;
        if *energy <= 0.0 {
            *energy = 0.0;
            commands.entity(entity).remove::<Shielded>();

            let shield = children
                .iter()
                .copied()
                .find(|c| shields.get(*c).is_ok())
                .expect("cannot find shield entity");

            commands.entity(shield).despawn_recursive();
        }
        println!("Player {:?} energy: {}", entity, *energy);
    } else if *energy > 0.0 {
        commands
            .entity(entity)
            .insert(Shielded)
            .with_children(|player| {
                player
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load("images/shield.png"),
                        transform: Transform::from_scale(Vec3::splat(PLAYER_SHIELD_SCALE))
                            .with_translation(Vec3::new(0.0, 0.0, 5.0)),
                        sprite: Sprite {
                            color: Color::rgba(1.0, 1.0, 1.0, 0.1),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Shield);
            });
    }
}

fn handle_player_shooting(
    commands: &mut Commands,
    entity: Entity,
    energy: &mut f32,
    velocity: &Vec2,
    transform: &Transform,
    shielded: bool,
    asset_server: &Res<AssetServer>,
) {
    const PROJECTILE_ENERGY_COST: f32 = 1.0;
    const PROJECTILE_SPEED: f32 = 12.0;

    if *energy >= PROJECTILE_ENERGY_COST && !shielded {
        *energy -= PROJECTILE_ENERGY_COST;
        println!("Player {:?} energy: {}", entity, *energy);

        commands
            .spawn()
            .insert(Projectile)
            .insert(Originator(entity))
            .insert(Velocity(
                ((transform.rotation * Vec3::new(0.0, PROJECTILE_SPEED, 0.0)).truncate()
                    + *velocity / 2.0)
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

const PLAYER_MAX_SPEED: f32 = 8.0;
const PLAYER_MAX_ANGULAR_VELOCITY: f32 = 0.075;

fn enforce_speed_limits(mut query: Query<(&mut Velocity, &mut AngularVelocity), With<Player>>) {
    for (mut velocity, mut angular_velocity) in query.iter_mut() {
        if velocity.0.length() > PLAYER_MAX_SPEED {
            velocity.0 = velocity.0.normalize() * PLAYER_MAX_SPEED;
        }

        angular_velocity.0 = angular_velocity
            .0
            .min(PLAYER_MAX_ANGULAR_VELOCITY)
            .max(-PLAYER_MAX_ANGULAR_VELOCITY);
    }
}

fn move_players(
    mut query: Query<
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

    for (mut velocity, mut angular_velocity, mut transform, shielded) in query.iter_mut() {
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
fn handle_collision(mut query: Query<(&Player, &mut Velocity, &mut Transform, &CollisionCircle)>) {
    const COLLISION_ITERATIONS: usize = 10;
    const COLLISION_MARGIN: f32 = 0.1;

    let mut found_collision = false;

    'check_all: for _ in 0..COLLISION_ITERATIONS {
        // Collide players against others
        let mut combinations = query.iter_combinations_mut();
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
        for (_, mut velocity, mut transform, collision) in query.iter_mut() {
            if transform.translation.x + collision.radius > LOGICAL_WIDTH {
                transform.translation.x = LOGICAL_WIDTH - collision.radius - COLLISION_MARGIN;
                velocity.0 = Vec2::new(0.0, Vec2::new(0.0, 1.0).dot(velocity.0));
                found_collision = true;
            }

            if transform.translation.y + collision.radius > LOGICAL_HEIGHT {
                transform.translation.y = LOGICAL_HEIGHT - collision.radius - COLLISION_MARGIN;
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

fn move_projectiles(mut query: Query<(&mut Transform, &Velocity), With<Projectile>>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0);
    }
}

fn detect_projectile_hits(
    mut commands: Commands,
    mut hp_entities: Query<(
        Entity,
        &mut HitPoints,
        &Transform,
        &CollisionCircle,
        Option<&Shielded>,
    )>,
    projectiles: Query<(Entity, &Originator, &Transform, &CollisionCircle)>,
) {
    for (projectile_entity, originator, projectile_transform, projectile_collision) in
        projectiles.iter()
    {
        let hit = hp_entities
            .iter_mut()
            .filter(|(p, _, _, _, _)| *p != originator.0)
            .map(|(p, h, t, c, s)| {
                (
                    p,
                    h,
                    (projectile_transform.translation - t.translation)
                        .truncate()
                        .length()
                        - c.radius
                        - projectile_collision.radius,
                    s,
                )
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
            .filter(|a| a.2 <= 0.0)
            .map(|a| (a.0, a.1, a.3));

        if let Some((entity, mut hp, shielded)) = hit {
            commands.entity(projectile_entity).despawn();

            if shielded.is_none() && hp.0 > 0 {
                hp.0 -= 1;
                println!("Player {:?} hp: {}", entity, hp.0);
                if hp.0 == 0 {
                    commands.entity(entity).insert(Dead);
                }
            }
        }
    }
}

#[derive(Component)]
struct SwimmingAnimation(Timer);

fn animate_swimming(
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

fn animate_eyes(
    players: Query<
        (Entity, &Transform, &Children),
        (With<Player>, Without<BodyPart>, Without<Dead>),
    >,
    orbs: Query<&EnergyOrb, Without<EnergyOrbRespawnTimer>>,
    mut body_parts: Query<(
        &mut Transform,
        &GlobalTransform,
        &BodyPart,
        Option<&Children>,
    )>,
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
