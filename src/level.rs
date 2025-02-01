use std::time::Duration;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_tweening::{
    component_animator_system, AnimationSystem, Animator, Lens, Targetable, Tween,
};

use crate::{
    arena::{ArenaSet, ArenaSize},
    game::{GameEntity, SpawnLevel},
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, LevelSet.after(ArenaSet))
            .add_observer(on_spawn_level)
            .add_systems(FixedUpdate, resize_view.in_set(LevelSet))
            .add_systems(
                Update,
                component_animator_system::<Projection>.in_set(AnimationSystem::AnimationUpdate),
            );
    }
}

struct ScalingModeLens {
    start: f32,
    end: f32,
}

impl Lens<Projection> for ScalingModeLens {
    fn lerp(&mut self, target: &mut dyn Targetable<Projection>, ratio: f32) {
        if let Projection::Orthographic(orthographic_projection) = target.target_mut() {
            if let ScalingMode::FixedVertical { viewport_height } =
                &mut orthographic_projection.scaling_mode
            {
                *viewport_height = self.start + (self.end - self.start) * ratio;
            }
        }
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
    camera_query: Query<(Entity, &Projection), With<LevelCamera>>,
    mut commands: Commands,
) {
    const OFFSET: f32 = 4.0;
    for arena_size in arena_query.iter() {
        let new_viewport_height = arena_size.0 as f32 + OFFSET;
        for (entity, projection) in camera_query.iter() {
            if let Projection::Orthographic(projection) = projection {
                if let ScalingMode::FixedVertical { viewport_height } = projection.scaling_mode {
                    commands.entity(entity).insert(Animator::new(Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_secs_f32(0.5),
                        ScalingModeLens {
                            start: viewport_height,
                            end: new_viewport_height,
                        },
                    )));
                }
            }
        }
    }
}
