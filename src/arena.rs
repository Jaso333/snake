use bevy::{color::palettes::tailwind, prelude::*};

use crate::{
    game::{GameEntity, SpawnLevel, UnitCubeMesh},
    level::Score,
};

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_level)
            .add_observer(on_add_wall)
            .add_systems(PreStartup, insert_wall_material)
            .add_systems(
                FixedUpdate,
                (expand_arena, resize_walls).chain().in_set(ArenaSet),
            );
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct ArenaSet;

#[derive(Resource)]
struct WallMaterial(Handle<StandardMaterial>);

#[derive(Component)]
#[require(GameEntity, Mesh3d, MeshMaterial3d<StandardMaterial>)]
struct Wall {
    direction: Dir3,
}

impl Wall {
    fn new(direction: Dir3) -> Self {
        Self { direction }
    }
}

#[derive(Component)]
#[require(GameEntity)]
pub struct ArenaSize(pub i32);

impl Default for ArenaSize {
    fn default() -> Self {
        Self(11)
    }
}

impl ArenaSize {
    pub fn half_size(&self) -> i32 {
        self.0 / 2
    }

    pub fn area(&self) -> i32 {
        self.0 * self.0
    }
}

fn on_spawn_level(_: Trigger<SpawnLevel>, mut commands: Commands) {
    commands.spawn(ArenaSize::default());
    commands.spawn(Wall::new(Dir3::NEG_X));
    commands.spawn(Wall::new(Dir3::X));
    commands.spawn(Wall::new(Dir3::NEG_Z));
    commands.spawn(Wall::new(Dir3::Z));
}

fn on_add_wall(
    trigger: Trigger<OnAdd, Wall>,
    wall_material: Res<WallMaterial>,
    unit_cube_mesh: Res<UnitCubeMesh>,
    mut query: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let (mut mesh, mut material) = query.get_mut(trigger.entity()).unwrap();
    mesh.0 = unit_cube_mesh.0.clone();
    material.0 = wall_material.0.clone();
}

fn insert_wall_material(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(WallMaterial(materials.add(StandardMaterial {
        base_color: Color::from(tailwind::SLATE_400),
        perceptual_roughness: 1.0,
        ..default()
    })));
}

fn resize_walls(
    arena_query: Query<&ArenaSize, Changed<ArenaSize>>,
    mut wall_query: Query<(&Wall, &mut Transform)>,
) {
    for arena_size in arena_query.iter() {
        for (wall, mut transform) in wall_query.iter_mut() {
            transform.translation =
                wall.direction.as_vec3() * (arena_size.half_size() as f32 + 1.0);

            let scale_dir = if wall.direction.x != 0.0 {
                Vec3::new(0.0, 0.0, 1.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            };

            transform.scale =
                (Vec3::ONE - scale_dir) + scale_dir * arena_size.0 as f32 + (scale_dir * 2.0);
        }
    }
}

fn expand_arena(
    score_query: Query<&Score, Changed<Score>>,
    mut arena_query: Query<&mut ArenaSize>,
) {
    for score in score_query.iter() {
        for mut arena_size in arena_query.iter_mut() {
            if score.0 >= arena_size.area() as u32 / 4 {
                //if score.0 >= 1 { // for testing
                arena_size.0 += 2;
            }
        }
    }
}
