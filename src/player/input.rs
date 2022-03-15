use bevy::prelude::*;

use super::Player;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum Action {
    MoveForward(Entity),
    TurnLeft(Entity),
    TurnRight(Entity),
    Shield(Entity),
    Shoot(Entity),
}

#[derive(Clone, Component, Copy)]
pub struct KeyMap {
    pub forward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub shoot: KeyCode,
}

pub(super) fn gather_player_input(
    mut actions: ResMut<Input<Action>>,
    keyboard: Res<Input<KeyCode>>,
    players: Query<(Entity, &KeyMap), With<Player>>,
) {
    actions.clear();

    for (player, keymap) in players.iter() {
        // XXX Clean these up when https://github.com/bevyengine/bevy/pull/4209 lands in a release.

        if keyboard.just_pressed(keymap.forward) {
            actions.press(Action::MoveForward(player));
        } else if keyboard.just_released(keymap.forward) {
            actions.release(Action::MoveForward(player));
        }

        if keyboard.just_pressed(keymap.shoot) {
            actions.press(Action::Shoot(player));
        } else if keyboard.just_released(keymap.shoot) {
            actions.release(Action::Shoot(player));
        }

        if keyboard.pressed(keymap.left) && keyboard.pressed(keymap.right) {
            actions.press(Action::Shield(player));

            if actions.pressed(Action::TurnLeft(player)) {
                actions.release(Action::TurnLeft(player));
            }

            if actions.pressed(Action::TurnRight(player)) {
                actions.release(Action::TurnRight(player));
            }
        } else if (keyboard.pressed(keymap.left) && keyboard.just_released(keymap.right))
            || (keyboard.just_released(keymap.left) && keyboard.pressed(keymap.right))
            || (keyboard.just_released(keymap.left) && keyboard.just_released(keymap.right))
        {
            actions.release(Action::Shield(player));
        }

        if !actions.pressed(Action::Shield(player)) {
            if keyboard.pressed(keymap.left) {
                actions.press(Action::TurnLeft(player));
            } else if keyboard.just_released(keymap.left) {
                actions.release(Action::TurnLeft(player));
            }

            if keyboard.pressed(keymap.right) {
                actions.press(Action::TurnRight(player));
            } else if keyboard.just_released(keymap.right) {
                actions.release(Action::TurnRight(player));
            }
        }
    }
}
