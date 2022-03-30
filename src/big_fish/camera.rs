use bevy::core_pipeline::RenderTargetClearColors;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::render_resource::{
    Extent3d, FilterMode, TextureDimension, TextureFormat, TextureUsages,
};

use super::BIG_FISH_LAYER;

#[derive(Component, Default)]
pub(super) struct BigFishCamera;

pub(super) const BIG_FISH_TEXTURE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 0xDEADBEEF02);

pub(super) fn setup_camera(
    mut commands: Commands,
    mut clear_colors: ResMut<RenderTargetClearColors>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: 950,
            height: 950,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

    image.sampler_descriptor.min_filter = FilterMode::Linear;

    let image_handle = images.set(BIG_FISH_TEXTURE, image);
    let render_target = RenderTarget::Image(image_handle.clone());

    clear_colors.insert(render_target.clone(), Color::rgba(0.0, 0.0, 0.0, 0.0));

    let bundle = OrthographicCameraBundle::new_2d();
    commands
        .spawn_bundle(OrthographicCameraBundle::<BigFishCamera> {
            camera: Camera {
                target: render_target,
                ..bundle.camera
            },
            orthographic_projection: OrthographicProjection {
                scaling_mode: ScalingMode::WindowSize,
                ..bundle.orthographic_projection
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            marker: BigFishCamera,
            visible_entities: bundle.visible_entities,
            frustum: bundle.frustum,
            global_transform: bundle.global_transform,
        })
        .insert(BIG_FISH_LAYER);
}
