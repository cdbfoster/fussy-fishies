#![allow(clippy::type_complexity)]

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::core_components::{
    AngularVelocity, CollisionCircle, Dead, Energy, HitPoints, Lives, Velocity,
};
use crate::State;

pub use self::input::KeyMap;
pub use self::model::PLAYER_SCALE;
pub use self::shield::PLAYER_SHIELD_SCALE;

use self::animation::{animate_eyes, animate_swimming, SwimmingAnimation};
use self::input::{gather_player_input, Action};
use self::model::{build_model, BodyPart};
use self::movement::{handle_collision, handle_movement, move_players};
use self::projectiles::{handle_projectiles, handle_shooting};
use self::shield::handle_shielding;

mod animation;
mod input;
mod model;
mod movement;
mod projectiles;
mod shield;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerConfiguration(vec![None; 4]))
            .insert_resource(Input::<Action>::default())
            .add_system_set(SystemSet::on_enter(State::Game).with_system(create_players))
            .add_system_set(
                SystemSet::on_update(State::Game)
                    .with_system(gather_player_input.before("input"))
                    .with_system(handle_movement.label("input"))
                    .with_system(handle_shielding.label("input"))
                    .with_system(handle_shooting.label("input"))
                    .with_system(
                        move_players
                            .label("move_players")
                            .label("physics")
                            .after("input"),
                    )
                    .with_system(
                        handle_collision
                            .label("handle_collision")
                            .label("physics")
                            .after("move_players"),
                    )
                    .with_system(
                        handle_projectiles
                            .label("physics")
                            .after("handle_collision"),
                    )
                    .with_system(
                        animate_swimming
                            .label("animate_swimming")
                            .label("animation")
                            .after("physics"),
                    )
                    .with_system(animate_eyes.label("animation").after("animate_swimming"))
                    .with_system(dying.after("animation")),
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

#[derive(Clone, Component)]
pub struct PlayerColor(pub Color);

#[derive(Bundle, Default)]
struct PlayerObjectBundle {
    velocity: Velocity,
    angular_velocity: AngularVelocity,
    energy: Energy,
}

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

    for (i, player_configuration) in player_config.0.iter().cloned().flatten().enumerate() {
        let mut player = commands.spawn();

        player
            .insert(Player)
            .insert_bundle(player_configuration.clone())
            .insert_bundle(PlayerObjectBundle::default())
            .insert(CollisionCircle {
                radius: 128.0 * PLAYER_SCALE,
            })
            .insert(SwimmingAnimation(Timer::from_seconds(0.333, true)));

        build_model(
            &mut player,
            &asset_server,
            Vec2::new(PLAYER_START_POSITIONS[i].0, PLAYER_START_POSITIONS[i].1),
            Quat::from_rotation_z(PLAYER_START_ANGLES[i]),
            player_configuration.color.0,
        );
    }
}
