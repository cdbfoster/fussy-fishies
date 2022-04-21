use bevy::core_pipeline::RenderTargetClearColors;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::camera::{RenderTarget, ScalingMode, WindowOrigin};
use bevy::render::render_resource::{
    Extent3d, FilterMode, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::RenderLayers;

use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};

#[derive(Component, Default)]
pub struct MainCamera;

pub fn spawn_main_camera<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
) -> EntityCommands<'w, 's, 'a> {
    let mut camera = commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            window_origin: WindowOrigin::BottomLeft,
            scaling_mode: ScalingMode::None,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..OrthographicCameraBundle::new_2d()
    });

    camera.insert(MainCamera).insert(RenderLayers::layer(1));

    camera
}

pub const FOREGROUND_COLOR_TEXTURE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 0xDEADBEEF01);

#[derive(Component, Default)]
pub struct ForegroundCamera;

pub fn spawn_foreground_camera<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    images: &mut ResMut<Assets<Image>>,
    clear_colors: &mut ResMut<RenderTargetClearColors>,
) -> EntityCommands<'w, 's, 'a> {
    let mut image = Image::new_fill(
        Extent3d {
            width: LOGICAL_WIDTH,
            height: LOGICAL_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

    image.sampler_descriptor.min_filter = FilterMode::Linear;

    let image_handle = images.set(FOREGROUND_COLOR_TEXTURE, image);
    let render_target = RenderTarget::Image(image_handle);

    clear_colors.insert(render_target.clone(), Color::rgba(0.0, 0.0, 0.0, 0.0));

    let bundle = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(OrthographicCameraBundle::<ForegroundCamera> {
        camera: Camera {
            target: render_target,
            ..bundle.camera
        },
        orthographic_projection: OrthographicProjection {
            window_origin: WindowOrigin::BottomLeft,
            scaling_mode: ScalingMode::None,
            left: 0.0,
            right: LOGICAL_WIDTH as f32,
            bottom: 0.0,
            top: LOGICAL_HEIGHT as f32,
            ..bundle.orthographic_projection
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        marker: ForegroundCamera,
        visible_entities: bundle.visible_entities,
        frustum: bundle.frustum,
        global_transform: bundle.global_transform,
    })
}

pub fn setup_cameras(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
) {
    spawn_main_camera(&mut commands);
    spawn_foreground_camera(&mut commands, &mut images, &mut clear_colors);
}
