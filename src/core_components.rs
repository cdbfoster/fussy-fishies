use bevy::prelude::*;

#[derive(Clone, Component, Default)]
pub struct AngularVelocity(pub f32);

#[derive(Clone, Component)]
pub struct CollisionCircle {
    pub radius: f32,
}

#[derive(Clone, Component)]
pub struct Dead;

#[derive(Clone, Component, Default)]
pub struct Energy(pub f32);

#[derive(Clone, Component)]
pub struct HitPoints(pub u32);

#[derive(Clone, Component)]
pub struct Lives(pub u32);

#[derive(Clone, Component)]
pub struct Originator(pub Entity);

#[derive(Clone, Component)]
pub struct Projectile;

#[derive(Clone, Component, Default)]
pub struct Shielded;

#[derive(Clone, Component, Default)]
pub struct Velocity(pub Vec2);
