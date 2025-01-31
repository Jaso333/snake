use bevy::{color::palettes::tailwind, prelude::*};
use rand::{thread_rng, Rng};

use crate::{
    arena::ArenaSize,
    game::{GameEntity, PostSpawnLevel},
    grid::GridPosition,
};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_food)
            .add_observer(on_spawn_food)
            .add_observer(on_post_spawn_level)
            .add_systems(PreStartup, (insert_food_material, insert_food_mesh));
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct FoodSet;

#[derive(Event)]
struct SpawnFood;

#[derive(Resource)]
struct FoodMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
struct FoodMesh(Handle<Mesh>);

#[derive(Component)]
#[require(GameEntity, Mesh3d, MeshMaterial3d<StandardMaterial>)]
struct Food;

fn on_add_food(
    trigger: Trigger<OnAdd, Food>,
    food_material: Res<FoodMaterial>,
    food_mesh: Res<FoodMesh>,
    mut query: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let (mut mesh, mut material) = query.get_mut(trigger.entity()).unwrap();
    mesh.0 = food_mesh.0.clone();
    material.0 = food_material.0.clone();
}

fn on_spawn_food(
    _: Trigger<SpawnFood>,
    grid_query: Query<&GridPosition>,
    arena_query: Query<&ArenaSize>,
    mut commands: Commands,
) {
    let Some(arena_size) = arena_query.iter().next() else {
        return;
    };

    let mut pool = Vec::with_capacity(arena_size.area() as usize);
    for x in -arena_size.half_size()..=arena_size.half_size() {
        for z in -arena_size.half_size()..=arena_size.half_size() {
            let grid_position = GridPosition(IVec3::new(x, 0, z));
            if grid_query.iter().all(|gp| gp != &grid_position) {
                pool.push(grid_position);
            }
        }
    }

    let mut rng = thread_rng();
    let grid_position = pool.swap_remove(rng.gen_range(0..pool.len()));
    commands.spawn((Food, grid_position));
}

fn on_post_spawn_level(_: Trigger<PostSpawnLevel>, mut commands: Commands) {
    commands.trigger(SpawnFood);
}

fn insert_food_material(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(FoodMaterial(materials.add(StandardMaterial {
        base_color: Color::from(tailwind::RED_500),
        perceptual_roughness: 1.0,
        ..default()
    })));
}

fn insert_food_mesh(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    commands.insert_resource(FoodMesh(meshes.add(Sphere::new(0.5))));
}
