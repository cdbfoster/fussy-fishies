use std::f32::consts::PI;
use std::ops::Range;

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use rand::{thread_rng, Rng};

use crate::configuration::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::State;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(BubbleTimer::default())
            .add_system_set(SystemSet::on_enter(State::Game).with_system(create_background))
            .add_system_set(
                SystemSet::on_update(State::Game)
                    .with_system(spawn_bubbles)
                    .with_system(move_bubbles)
                    .with_system(despawn_bubbles),
            );
    }
}

fn create_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/background.png"),
            transform: Transform::from_translation(Vec3::new(
                LOGICAL_WIDTH as f32 / 2.0,
                LOGICAL_HEIGHT as f32 / 2.0,
                -1.0,
            )),
            ..default()
        })
        .insert(RenderLayers::layer(1));
}

const BUBBLE_GROUP_TIMER_RANGE: Range<f32> = 1.5..6.0;
const BUBBLE_GROUP_COUNT_RANGE: Range<usize> = 4..8;
const BUBBLE_GROUP_Z_RANGE: Range<f32> = -10.0..-1.0;
const BUBBLE_X_RANGE: Range<f32> = -35.0..35.0;
const BUBBLE_Y_RANGE: Range<f32> = 0.0..150.0;
const BUBBLE_Z_RANGE: Range<f32> = -1.0..1.0;
const BUBBLE_SIZE_RANGE: Range<f32> = 5.0..20.0;

#[derive(Default)]
struct BubbleTimer(Timer);

#[derive(Component)]
struct Bubble;

#[derive(Component)]
struct Wobble {
    phase: f32,
    amplitude: f32,
}

pub fn spawn_bubble_group(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    g_pos: Vec3,
    count: usize,
    x_range: Range<f32>,
    y_range: Range<f32>,
    z_range: Range<f32>,
) {
    let mut rng = thread_rng();

    for _ in 0..count {
        let ox = rng.gen_range(x_range.clone());
        let oy = rng.gen_range(y_range.clone());
        let oz = rng.gen_range(z_range.clone());

        let size = rng.gen_range(BUBBLE_SIZE_RANGE) / 256.0;

        let alpha = ((11.0 + g_pos.z + oz) / 33.0 + 0.3).min(1.0).max(0.0);

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, alpha),
                    ..default()
                },
                texture: asset_server.load("images/bubble-small.png"),
                transform: Transform::from_translation(Vec3::new(
                    g_pos.x + ox,
                    g_pos.y + oy,
                    g_pos.z + oz,
                ))
                .with_scale(Vec3::splat(size)),
                ..default()
            })
            .insert(Bubble)
            .insert(Wobble {
                phase: rng.gen_range(0.0..2.0 * PI),
                amplitude: 5.0 * size,
            });
    }
}

fn spawn_bubbles(
    mut commands: Commands,
    mut timer: ResMut<BubbleTimer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let mut rng = thread_rng();

        spawn_bubble_group(
            &mut commands,
            &asset_server,
            Vec3::new(
                rng.gen_range(0.0..LOGICAL_WIDTH as f32),
                -BUBBLE_Y_RANGE.end - BUBBLE_SIZE_RANGE.end / 2.0,
                rng.gen_range(BUBBLE_GROUP_Z_RANGE),
            ),
            rng.gen_range(BUBBLE_GROUP_COUNT_RANGE),
            BUBBLE_X_RANGE,
            BUBBLE_Y_RANGE,
            BUBBLE_Z_RANGE,
        );

        timer.0 = Timer::from_seconds(thread_rng().gen_range(BUBBLE_GROUP_TIMER_RANGE), false);
    }
}

fn move_bubbles(time: Res<Time>, mut bubbles: Query<(&mut Transform, &Wobble), With<Bubble>>) {
    const BUBBLE_VELOCITY: f32 = 1.5;
    const BUBBLE_VELOCITY_SIZE_BONUS: f32 = 0.05;

    for (mut transform, wobble) in bubbles.iter_mut() {
        let size = transform.scale.z * 256.0;
        let size_bonus = (size - BUBBLE_SIZE_RANGE.start) * BUBBLE_VELOCITY_SIZE_BONUS;

        transform.translation.x += (time.seconds_since_startup() * 16.0 + wobble.phase as f64).sin()
            as f32
            * wobble.amplitude;
        transform.translation.y += BUBBLE_VELOCITY + size_bonus;
    }
}

fn despawn_bubbles(mut commands: Commands, bubbles: Query<(Entity, &Transform), With<Bubble>>) {
    for (entity, transform) in bubbles.iter() {
        if transform.translation.y > LOGICAL_HEIGHT as f32 {
            commands.entity(entity).despawn();
        }
    }
}
