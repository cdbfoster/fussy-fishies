use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use self::background::BackgroundPlugin;
use self::configuration::ConfigurationPlugin;
use self::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use self::core_components::{HitPoints, Lives};
use self::energy_orbs::EnergyOrbsPlugin;
use self::player::{
    KeyMap, PlayerColor, PlayerConfiguration, PlayerConfigurationBundle, PlayerPlugin,
};
use self::render::additional_pass::AdditionalPassPlugin;
use self::render::cameras::{setup_cameras, ForegroundCamera, FOREGROUND_COLOR_TEXTURE};

mod animation;
mod background;
mod configuration;
mod core_components;
mod energy_orbs;
mod player;
mod render;

fn main() {
    App::new()
        .add_plugin(ConfigurationPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(BackgroundPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnergyOrbsPlugin)
        .add_plugin(AdditionalPassPlugin::<ForegroundCamera>::new(
            "foreground_pass",
            None,
        ))
        .add_state(State::Game)
        .add_startup_system(setup_cameras)
        .add_startup_system(setup)
        .run();
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum State {
    Menu,
    Game,
}

fn setup(
    mut commands: Commands,
    mut player_config: ResMut<PlayerConfiguration>,
    images: Res<Assets<Image>>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: images.get_handle(FOREGROUND_COLOR_TEXTURE),
            transform: Transform::from_translation(Vec3::new(
                LOGICAL_WIDTH as f32 / 2.0,
                LOGICAL_HEIGHT as f32 / 2.0,
                0.0,
            )),
            ..Default::default()
        })
        .insert(RenderLayers::layer(1));

    const DEFAULT_PLAYER_KEY_MAPS: [KeyMap; 4] = [
        KeyMap {
            forward: KeyCode::W,
            left: KeyCode::A,
            right: KeyCode::D,
            shoot: KeyCode::S,
        },
        KeyMap {
            forward: KeyCode::I,
            left: KeyCode::J,
            right: KeyCode::L,
            shoot: KeyCode::K,
        },
        KeyMap {
            forward: KeyCode::Up,
            left: KeyCode::Left,
            right: KeyCode::Right,
            shoot: KeyCode::Down,
        },
        KeyMap {
            forward: KeyCode::Numpad8,
            left: KeyCode::Numpad4,
            right: KeyCode::Numpad6,
            shoot: KeyCode::Numpad5,
        },
    ];

    const DEFAULT_PLAYER_COLORS: [Color; 4] = [
        Color::rgb(1.0, 0.65, 0.65),
        Color::rgb(0.85, 0.65, 1.0),
        Color::rgb(0.65, 0.9, 0.65),
        Color::rgb(1.0, 1.0, 0.65),
    ];

    for (i, player) in player_config.0.iter_mut().enumerate().take(4) {
        *player = Some(PlayerConfigurationBundle {
            keymap: DEFAULT_PLAYER_KEY_MAPS[i],
            color: PlayerColor(DEFAULT_PLAYER_COLORS[i]),
            hp: HitPoints(5),
            lives: Lives(3),
        })
    }
}
