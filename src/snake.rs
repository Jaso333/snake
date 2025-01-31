use bevy::{color::palettes::tailwind, prelude::*};

use crate::{
    arena::{ArenaSet, ArenaSize},
    game::{GameEntity, SpawnLevel, UnitCubeMesh},
    grid::{GridPosition, GridSet},
};

pub struct SnakePlugin;

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, SnakeSet.after(ArenaSet).before(GridSet))
            .add_observer(on_spawn_level)
            .add_observer(on_add_snake_visual)
            .add_systems(PreStartup, insert_snake_material)
            .add_systems(Update, control_snake)
            .add_systems(FixedUpdate, move_snake.in_set(SnakeSet));
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct SnakeSet;

#[derive(Event)]
pub struct SnakeCollided;

#[derive(Resource)]
struct SnakeMaterial(Handle<StandardMaterial>);

#[derive(Component)]
#[require(
    GameEntity,
    SnakeVisual,
    SnakeMoveTimer,
    SnakeDirection,
    SnakeBodyBuffer,
    GridPosition
)]
struct SnakeHead;

#[derive(Component)]
#[require(GameEntity, SnakeVisual)]
struct SnakeBodySegment;

#[derive(Component, Default)]
#[require(Mesh3d, MeshMaterial3d<StandardMaterial>)]
struct SnakeVisual;

#[derive(Component)]
struct SnakeMoveTimer(Timer);

impl Default for SnakeMoveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.3, TimerMode::Repeating))
    }
}

#[derive(Component)]
struct SnakeDirection(Dir3);

impl Default for SnakeDirection {
    fn default() -> Self {
        Self(Dir3::NEG_Z)
    }
}

#[derive(Component)]
struct SnakeBodyBuffer(usize);

#[derive(Component, PartialEq, Eq, PartialOrd, Ord)]
struct SnakeBodyIndex(u32);

impl Default for SnakeBodyBuffer {
    fn default() -> Self {
        Self(2)
    }
}

fn on_spawn_level(_: Trigger<SpawnLevel>, mut commands: Commands) {
    commands.spawn(SnakeHead);
}

fn on_add_snake_visual(
    trigger: Trigger<OnAdd, SnakeVisual>,
    snake_material: Res<SnakeMaterial>,
    unit_cube_mesh: Res<UnitCubeMesh>,
    mut query: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let (mut mesh, mut material) = query.get_mut(trigger.entity()).unwrap();
    mesh.0 = unit_cube_mesh.0.clone();
    material.0 = snake_material.0.clone();
}

fn insert_snake_material(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(SnakeMaterial(materials.add(StandardMaterial {
        base_color: Color::from(tailwind::GREEN_500),
        perceptual_roughness: 1.0,
        ..default()
    })));
}

fn control_snake(
    mut query: Query<(&mut SnakeDirection, &mut SnakeMoveTimer)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (mut direction, mut timer) in query.iter_mut() {
        let mut input_direction = None;
        if input.just_pressed(KeyCode::KeyA) {
            input_direction = Some(Dir3::NEG_X);
        } else if input.just_pressed(KeyCode::KeyD) {
            input_direction = Some(Dir3::X);
        } else if input.just_pressed(KeyCode::KeyW) {
            input_direction = Some(Dir3::NEG_Z);
        } else if input.just_pressed(KeyCode::KeyS) {
            input_direction = Some(Dir3::Z);
        }

        // stop here if no input
        let Some(input_direction) = input_direction else {
            continue;
        };

        // don't do anything if trying to 180 the snake
        if input_direction
            == match direction.0 {
                Dir3::NEG_X => Dir3::X,
                Dir3::X => Dir3::NEG_X,
                Dir3::NEG_Z => Dir3::Z,
                Dir3::Z => Dir3::NEG_Z,
                _ => continue,
            }
        {
            continue;
        }

        // set the new direction
        direction.0 = input_direction;

        // immediately move - this give more natural input feel
        let duration = timer.0.duration();
        timer.0.set_elapsed(duration);
    }
}

fn move_snake(
    mut head_query: Query<
        (
            &SnakeDirection,
            &mut SnakeMoveTimer,
            &mut GridPosition,
            &mut SnakeBodyBuffer,
        ),
        Without<SnakeBodyIndex>,
    >,
    mut body_query: Query<(&mut SnakeBodyIndex, &mut GridPosition)>,
    arena_query: Query<&ArenaSize>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if arena_query.is_empty() {
        return;
    }

    let arena_size = arena_query.single();

    for (direction, mut timer, mut grid_position, mut buffer) in head_query.iter_mut() {
        if !timer.0.tick(time.delta()).just_finished() {
            continue;
        }

        // move the head forward by the snake's direction, detecting arena bounds collision
        let prev_position = grid_position.0;
        let next_position = grid_position.0 + direction.0.as_ivec3();

        // check the next position for a wall or any other snake part
        if [next_position.x, next_position.z]
            .iter()
            .any(|e| e > &arena_size.half_size() || e < &-arena_size.half_size())
            || body_query.iter().any(|(_, gp)| gp.0 == next_position)
        {
            timer.0.pause();
            commands.trigger(SnakeCollided);
            continue;
        }
        grid_position.0 += direction.0.as_ivec3();

        // set the defaults for the next body piece
        let mut next_body_index = 0;
        let mut next_body_position = prev_position;

        // get the current max index
        if let Some((max_index, _)) = body_query.iter().max_by(|(i1, ..), (i2, ..)| i1.cmp(i2)) {
            let max_index = max_index.0;

            // find the segment at the back of the snake
            if let Some((mut min_index, mut min_grid_position)) = body_query
                .iter_mut()
                .min_by(|(i1, ..), (i2, ..)| i1.cmp(i2))
            {
                // move the body segment from the back to the head's previous position
                next_body_position = min_grid_position.0;
                next_body_index = min_index.0;
                min_index.0 = max_index + 1;
                min_grid_position.0 = prev_position;
            }
        }

        if buffer.0 == 0 {
            continue;
        }

        buffer.0 -= 1;

        // spawn the next body segment to fill the moved segment
        commands.spawn((
            SnakeBodySegment,
            SnakeBodyIndex(next_body_index),
            GridPosition(next_body_position),
        ));
    }
}
