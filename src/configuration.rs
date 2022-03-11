use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResized};

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

pub const LOGICAL_WIDTH: f32 = 1920.0;
pub const LOGICAL_HEIGHT: f32 = 1080.0;
pub const LOGICAL_ASPECT: f32 = 16.0 / 9.0;

fn adjust_projections(mut query: Query<&mut OrthographicProjection>, windows: Res<Windows>, mut resized: EventReader<WindowResized>) {
    for resized_event in resized.iter() {
        let window = windows.get(resized_event.id).expect("cannot get window");

        let (width, height) = (window.width(), window.height());

        let (size_x, size_y) = if width >= height * LOGICAL_ASPECT {
            (LOGICAL_HEIGHT * width / height, LOGICAL_HEIGHT)
        } else {
            (LOGICAL_WIDTH, LOGICAL_WIDTH * height / width)
        };

        let (offset_x, offset_y) = if width >= height * LOGICAL_ASPECT {
            ((LOGICAL_WIDTH - size_x) / 2.0, 0.0)
        } else {
            (0.0, (LOGICAL_HEIGHT - size_y) / 2.0)
        };

        for mut projection in query.iter_mut() {
            projection.left = offset_x;
            projection.right = size_x + offset_x;
            projection.bottom = offset_y;
            projection.top = size_y + offset_y;
        }
    }
}