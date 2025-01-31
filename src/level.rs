use bevy::{prelude::*, render::camera::ScalingMode};

use crate::game::{GameEntity, SpawnLevel};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_level);
    }
}

#[derive(Component)]
#[require(
    GameEntity,
    Camera3d,
    Projection(Self::projection),
    Transform(Self::transform)
)]
struct LevelCamera;

impl LevelCamera {
    fn projection() -> Projection {
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 16.0,
            },
            ..OrthographicProjection::default_2d()
        })
    }

    fn transform() -> Transform {
        Transform::from_xyz(0.5, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y)
    }
}

#[derive(Component)]
#[require(GameEntity, DirectionalLight, Transform(Self::transform))]
struct LevelLight;

impl LevelLight {
    fn transform() -> Transform {
        Transform::from_xyz(0.75, 2.0, 0.5).looking_at(Vec3::ZERO, Vec3::Y)
    }
}

fn on_spawn_level(_: Trigger<SpawnLevel>, mut commands: Commands) {
    commands.spawn(LevelCamera);
    commands.spawn(LevelLight);
}
