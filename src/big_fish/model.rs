use std::f32::consts::PI;

use bevy::prelude::*;

use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};

use super::camera::BIG_FISH_TEXTURE;
use super::{BigFish, BIG_FISH_LAYER, START_DEPTH, START_SCALE};

#[derive(Clone, Component, Copy, Debug, PartialEq)]
pub(super) enum BodyPart {
    Head,
    TopJaw,
    BottomJaw,
    Lure,
    RightFin,
    LeftFin,
}

pub(super) fn build_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/big_fish/head.png"),
            transform: Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
            ..default()
        })
        .insert(BodyPart::Head)
        .insert(BIG_FISH_LAYER)
        .with_children(|head| {
            head.spawn_bundle(SpriteBundle {
                texture: asset_server.load("images/big_fish/jaw-top-teeth.png"),
                transform: Transform::from_translation(Vec3::new(0.0, -30.0, 2.0)),
                ..default()
            })
            .insert(BodyPart::TopJaw)
            .insert(BIG_FISH_LAYER)
            .with_children(|jaw| {
                jaw.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("images/big_fish/jaw-top-gums.png"),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                    ..default()
                })
                .insert(BIG_FISH_LAYER);
            });

            head.spawn_bundle(SpriteBundle {
                texture: asset_server.load("images/big_fish/jaw-bottom-teeth.png"),
                transform: Transform::from_translation(Vec3::new(0.0, 30.0, 1.5)),
                ..default()
            })
            .insert(BodyPart::BottomJaw)
            .insert(BIG_FISH_LAYER)
            .with_children(|jaw| {
                jaw.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("images/big_fish/jaw-bottom-gums.png"),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                    ..default()
                })
                .insert(BIG_FISH_LAYER);
            });

            head.spawn_bundle(SpriteBundle {
                texture: asset_server.load("images/big_fish/lure-stalk.png"),
                transform: Transform::from_translation(Vec3::new(0.0, 550.0, 1.0)),
                ..default()
            })
            .insert(BIG_FISH_LAYER)
            .with_children(|stalk| {
                stalk
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load("images/big_fish/lure.png"),
                        ..default()
                    })
                    .insert(BodyPart::Lure)
                    .insert(BIG_FISH_LAYER);
            });

            head.spawn_bundle(SpriteBundle {
                texture: asset_server.load("images/big_fish/fin.png"),
                transform: Transform::from_translation(Vec3::new(315.0, 0.0, -1.0))
                    .with_rotation(Quat::from_rotation_z(30.0 * PI / 180.0)),
                ..default()
            })
            .insert(BodyPart::RightFin)
            .insert(BIG_FISH_LAYER);

            head.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    flip_x: true,
                    ..default()
                },
                texture: asset_server.load("images/big_fish/fin.png"),
                transform: Transform::from_translation(Vec3::new(-315.0, 0.0, -1.0))
                    .with_rotation(Quat::from_rotation_z(-30.0 * PI / 180.0)),
                ..default()
            })
            .insert(BodyPart::LeftFin)
            .insert(BIG_FISH_LAYER);
        });

    let image_handle = images.get_handle(BIG_FISH_TEXTURE);

    commands
        .spawn_bundle(SpriteBundle {
            texture: image_handle,
            transform: Transform::from_translation(Vec3::new(
                LOGICAL_WIDTH as f32 / 2.0,
                LOGICAL_HEIGHT as f32 / 2.0,
                START_DEPTH,
            ))
            .with_scale(Vec3::splat(START_SCALE)),
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 0.3),
                ..default()
            },
            ..default()
        })
        .insert(BigFish);
}
