use bevy::prelude::*;
use bevy::render::camera::{ScalingMode, WindowOrigin};
use bevy::window::WindowMode;

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
            .add_startup_system(setup_camera);
    }
}

pub const LOGICAL_WIDTH: f32 = 1920.0;
pub const LOGICAL_HEIGHT: f32 = 1080.0;
pub const LOGICAL_ASPECT: f32 = 16.0 / 9.0;

fn setup_camera(mut commands: Commands, windows: Res<Windows>) {
    let window = windows.get_primary().expect("cannot get primary window");

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

    commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            left: offset_x,
            right: size_x + offset_x,
            bottom: offset_y,
            top: size_y + offset_y,
            near: 0.0,
            far: 1000.0,
            window_origin: WindowOrigin::BottomLeft,
            scaling_mode: ScalingMode::None,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0))
            .looking_at(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0)),
        ..OrthographicCameraBundle::new_2d()
    });
}
