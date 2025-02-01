use bevy::{color::palettes::tailwind, prelude::*};
use rand::{thread_rng, Rng};

use crate::{
    arena::ArenaSize,
    game::GameEntity,
    grid::{GridPosition, GridSet},
    level::Score,
    snake::{SnakeBodyBuffer, SnakeHead, SnakeSet},
};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, FoodSet.after(SnakeSet).before(GridSet))
            .add_observer(on_add_food)
            .add_systems(PreStartup, (insert_food_material, insert_food_mesh))
            .add_systems(FixedUpdate, (eat_food, spawn_food).chain().in_set(FoodSet));
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct FoodSet;

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

fn eat_food(
    mut snake_query: Query<
        (&GridPosition, &mut SnakeBodyBuffer),
        (With<SnakeHead>, Changed<GridPosition>),
    >,
    food_query: Query<(Entity, &GridPosition), (With<Food>, Without<SnakeHead>)>,
    mut score_query: Query<&mut Score>,
    mut commands: Commands,
) {
    for (snake_grid_position, mut buffer) in snake_query.iter_mut() {
        for food_entity in food_query
            .iter()
            .filter_map(|(e, gp)| (gp == snake_grid_position).then(|| e))
        {
            commands.entity(food_entity).despawn_recursive();
            buffer.0 += 1; // extend the body

            // increment the score
            for mut score in score_query.iter_mut() {
                score.0 += 1;
            }
        }
    }
}

fn spawn_food(
    grid_query: Query<&GridPosition>,
    arena_query: Query<&ArenaSize>,
    food_query: Query<&Food>,
    snake_query: Query<&SnakeHead>,
    mut commands: Commands,
) {
    if !food_query.is_empty() || snake_query.is_empty() {
        return;
    }

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
