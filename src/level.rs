use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{
    arena::{ArenaSet, ArenaSize},
    game::{GameEntity, SpawnLevel},
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, LevelSet.after(ArenaSet))
            .add_observer(on_spawn_level)
            .add_systems(FixedUpdate, resize_view.in_set(LevelSet));
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct LevelSet;

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

#[derive(Component, Default)]
#[require(GameEntity)]
pub struct Score(pub u32);

fn on_spawn_level(_: Trigger<SpawnLevel>, mut commands: Commands) {
    commands.spawn(LevelCamera);
    commands.spawn(LevelLight);
    commands.spawn(Score::default());
}

fn resize_view(
    arena_query: Query<&ArenaSize, Changed<ArenaSize>>,
    mut camera_query: Query<&mut Projection, With<LevelCamera>>,
) {
    const OFFSET: f32 = 4.0;
    for arena_size in arena_query.iter() {
        let viewport_height = arena_size.0 as f32 + OFFSET;
        for projection in camera_query.iter_mut() {
            if let Projection::Orthographic(projection) = projection.into_inner() {
                projection.scaling_mode = ScalingMode::FixedVertical { viewport_height };
            }
        }
    }
}
