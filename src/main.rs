use bevy::prelude::*;

mod animation;
mod configuration;
mod core_components;
mod energy_orbs;
mod player;

use self::configuration::ConfigurationPlugin;
use self::core_components::{HitPoints, Lives};
use self::energy_orbs::EnergyOrbsPlugin;
use self::player::{
    KeyMap, PlayerColor, PlayerConfiguration, PlayerConfigurationBundle, PlayerPlugin,
};

fn main() {
    App::new()
        .add_plugin(ConfigurationPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnergyOrbsPlugin)
        .add_state(State::Game)
        .add_startup_system(setup)
        .run();
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum State {
    Menu,
    Game,
}

fn setup(mut player_config: ResMut<PlayerConfiguration>) {
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
        Color::rgb(1.0, 0.8, 0.8),
        Color::rgb(0.8, 0.8, 1.0),
        Color::rgb(0.8, 1.0, 0.8),
        Color::rgb(1.0, 1.0, 0.8),
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
