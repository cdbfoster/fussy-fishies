use bevy::prelude::*;

use crate::background::spawn_bubble_group;
use crate::core_components::{Energy, Shield, Shielded};

use super::input::Action;
use super::Player;

pub const PLAYER_SHIELD_SCALE: f32 = 1.2;

pub(super) fn handle_shielding(
    mut commands: Commands,
    time: Res<Time>,
    actions: Res<Input<Action>>,
    asset_server: Res<AssetServer>,
    mut players: Query<
        (
            Entity,
            &mut Energy,
            &Transform,
            &Children,
            Option<&Shielded>,
        ),
        With<Player>,
    >,
    shields: Query<(), With<Shield>>,
) {
    const SHIELD_DRAIN_RATE: f32 = 2.0;

    for (player, mut energy, transform, children, shielded) in players.iter_mut() {
        if actions.just_pressed(Action::Shield(player)) && energy.0 > 0.0 {
            commands
                .entity(player)
                .insert(Shielded)
                .with_children(|player| {
                    player
                        .spawn_bundle(SpriteBundle {
                            texture: asset_server.load("images/shield.png"),
                            transform: Transform::from_scale(Vec3::splat(PLAYER_SHIELD_SCALE))
                                .with_translation(Vec3::new(0.0, 0.0, 5.0)),
                            sprite: Sprite {
                                color: Color::rgba(1.0, 1.0, 1.0, 0.1),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(Shield);
                });
        } else if shielded.is_some() && actions.pressed(Action::Shield(player)) {
            energy.0 -= time.delta_seconds() * SHIELD_DRAIN_RATE;
            println!("Player {:?} energy: {}", player, energy.0);
        }

        if shielded.is_some() && (actions.just_released(Action::Shield(player)) || energy.0 <= 0.0)
        {
            if energy.0 < 0.0 {
                energy.0 = 0.0;
            }

            commands.entity(player).remove::<Shielded>();

            let shield = children
                .iter()
                .copied()
                .find(|c| shields.get(*c).is_ok())
                .expect("cannot find shield entity");

            commands.entity(shield).despawn_recursive();

            let range = (-128.0 * transform.scale.z)..(128.0 * transform.scale.z);

            spawn_bubble_group(
                &mut commands,
                &asset_server,
                transform.translation,
                5,
                range.clone(),
                range,
                5.0..5.0001,
            );
        }
    }
}
