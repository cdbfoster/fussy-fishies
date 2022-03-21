use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResized};

use crate::render::cameras::MainCamera;

pub struct ConfigurationPlugin;

impl Plugin for ConfigurationPlugin {
    fn build(&self, app: &mut App) {
        let window_configuration = WindowDescriptor {
            width: 1280.0,
            height: 720.0,
            resizable: false,
            cursor_visible: false,
            mode: WindowMode::Windowed,
            title: "Fishies".to_owned(),
            ..Default::default()
        };

        app.insert_resource(window_configuration)
            .add_system(adjust_projections);
    }
}

pub const LOGICAL_WIDTH: u32 = 1920;
pub const LOGICAL_HEIGHT: u32 = 1080;
pub const LOGICAL_ASPECT: f32 = 16.0 / 9.0;

fn adjust_projections(
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
    windows: Res<Windows>,
    mut resized: EventReader<WindowResized>,
) {
    for resized_event in resized.iter() {
        let window = windows.get(resized_event.id).expect("cannot get window");

        let (width, height) = (
            window.physical_width() as f32,
            window.physical_height() as f32,
        );

        let (size_x, size_y) = if width >= height * LOGICAL_ASPECT {
            (
                LOGICAL_HEIGHT as f32 * width / height,
                LOGICAL_HEIGHT as f32,
            )
        } else {
            (LOGICAL_WIDTH as f32, LOGICAL_WIDTH as f32 * height / width)
        };

        let (offset_x, offset_y) = if width >= height * LOGICAL_ASPECT {
            ((LOGICAL_WIDTH as f32 - size_x) / 2.0, 0.0)
        } else {
            (0.0, (LOGICAL_HEIGHT as f32 - size_y) / 2.0)
        };

        for mut projection in query.iter_mut() {
            projection.left = offset_x;
            projection.right = size_x + offset_x;
            projection.bottom = offset_y;
            projection.top = size_y + offset_y;
        }
    }
}
