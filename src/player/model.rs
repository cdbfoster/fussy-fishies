use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use super::PLAYER_SCALE;

#[derive(Clone, Component, Copy, PartialEq)]
pub(super) enum BodyPart {
    Head,
    Body,
    Tail,
    RightFin,
    LeftFin,
    RightEye,
    LeftEye,
}

pub(super) fn build_model(
    commands: &mut EntityCommands,
    asset_server: &Res<AssetServer>,
    start_position: Vec2,
    start_rotation: Quat,
    color: Color,
) {
    commands
        .insert_bundle(SpriteBundle {
            texture: asset_server.load("images/player/root.png"),
            transform: Transform::from_scale(Vec3::splat(PLAYER_SCALE))
                .with_translation(start_position.extend(1.0))
                .with_rotation(start_rotation),
            ..Default::default()
        })
        .with_children(|root| {
            root.spawn_bundle(SpriteBundle {
                texture: asset_server.load("images/player/head.png"),
                transform: Transform::from_translation(Vec3::new(0.0, 20.0, 1.0)),
                sprite: Sprite {
                    color,
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
                        color,
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
                        color,
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
                        color,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BodyPart::RightEye);

                head.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("images/player/eye-open.png"),
                    transform: Transform::from_translation(Vec3::new(-85.0, 62.0, 1.0)),
                    sprite: Sprite {
                        color,
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
                        color,
                        ..Default::default()
                    },
                    ..Default::default()
                });

                head.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("images/player/dorsal_fin.png"),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
                    sprite: Sprite {
                        color,
                        ..Default::default()
                    },
                    ..Default::default()
                });
            });

            root.spawn_bundle(SpriteBundle {
                texture: asset_server.load("images/player/body.png"),
                transform: Transform::from_translation(Vec3::new(0.0, -60.0, 2.0)),
                sprite: Sprite {
                    color,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BodyPart::Body);

            root.spawn_bundle(SpriteBundle {
                texture: asset_server.load("images/player/tail.png"),
                transform: Transform::from_translation(Vec3::new(0.0, -105.0, 3.0)),
                sprite: Sprite {
                    color,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BodyPart::Tail);
        });
}
