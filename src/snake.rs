use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    arena::{ArenaSet, ArenaSize},
    game::{AppExt, GameEntity, SpawnLevel},
    grid::{GridPosition, GridSet},
};

pub struct SnakePlugin;

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        app.add_game_assets::<SnakeAssets>()
            .configure_sets(FixedUpdate, SnakeSet.after(ArenaSet).before(GridSet))
            .add_observer(on_spawn_level)
            .add_observer(on_add_snake_head)
            .add_systems(Update, control_snake)
            .add_systems(
                FixedUpdate,
                (move_snake, (visualise_snake_head, visualise_snake_body))
                    .chain()
                    .in_set(SnakeSet),
            );
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct SnakeSet;

#[derive(Event)]
pub struct SnakeCollided;

#[derive(Resource, AssetCollection)]
struct SnakeAssets {
    #[asset(path = "body_straight.glb#Scene0")]
    body_straight: Handle<Scene>,
    #[asset(path = "body_corner.glb#Scene0")]
    body_corner: Handle<Scene>,
    #[asset(path = "body_end.glb#Scene0")]
    body_end: Handle<Scene>,
    #[asset(path = "head.glb#Scene0")]
    head: Handle<Scene>,
}

#[derive(Component)]
#[require(
    GameEntity,
    SnakeVisual,
    SnakeMoveTimer,
    SnakeDirection,
    SnakeBodyBuffer,
    GridPosition
)]
pub struct SnakeHead;

#[derive(Component)]
#[require(GameEntity, SnakeVisual)]
struct SnakeBodySegment;

#[derive(Component, Default)]
#[require(SceneRoot)]
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
pub struct SnakeBodyBuffer(pub usize);

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

fn on_add_snake_head(
    trigger: Trigger<OnAdd, SnakeHead>,
    mut query: Query<&mut SceneRoot>,
    assets: Res<SnakeAssets>,
) {
    query.get_mut(trigger.entity()).unwrap().0 = assets.head.clone();
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
    mut body_query: Query<(&SnakeBodyIndex, &mut GridPosition)>,
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
        let mut prev_position = grid_position.0;
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

        // shift all body segments forward
        let mut last_index = 0;
        for (index, mut grid_position) in body_query.iter_mut().sort::<&SnakeBodyIndex>() {
            let temp_grid_position = grid_position.0;
            grid_position.0 = prev_position;
            prev_position = temp_grid_position;
            last_index = index.0;
        }

        if buffer.0 == 0 {
            continue;
        }

        buffer.0 -= 1;

        // spawn the next body segment to fill the last spot
        commands.spawn((
            SnakeBodySegment,
            SnakeBodyIndex(last_index + 1),
            GridPosition(prev_position),
        ));
    }
}

fn visualise_snake_head(
    mut query: Query<(&SnakeDirection, &mut Transform), Changed<GridPosition>>,
) {
    for (direction, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(match direction.0 {
            Dir3::NEG_Z => 0.0,
            Dir3::Z => PI,
            Dir3::NEG_X => PI * 0.5,
            _ => PI * 1.5,
        });
    }
}

fn visualise_snake_body(
    mut body_query: Query<(
        Entity,
        Ref<GridPosition>,
        &SnakeBodyIndex,
        &mut Transform,
        &mut SceneRoot,
    )>,
    head_query: Query<&GridPosition, With<SnakeHead>>,
    assets: Option<Res<SnakeAssets>>,
) {
    let (Some(assets), Ok(head_grid_position)) = (assets, head_query.get_single()) else {
        return;
    };

    if body_query.iter().all(|(_, gp, ..)| !gp.is_changed()) {
        return;
    }

    // need to order the entities and look around them to calculate everything we need
    let ordered_entities: Vec<_> = body_query
        .iter()
        .sort::<&SnakeBodyIndex>()
        .rev()
        .map(|(e, ..)| e)
        .collect();

    for i in 0..ordered_entities.len() {
        let entity = ordered_entities[i];
        let (_, grid_position, ..) = body_query.get(entity).unwrap();
        let next_grid_position = match i + 1 < ordered_entities.len() {
            true => body_query.get(ordered_entities[i + 1]).unwrap().1.clone(),
            false => *head_grid_position,
        };

        // determine the direction to the next and previous segement
        let forward_direction = grid_direction(&grid_position, &next_grid_position);
        let back_direction = match i {
            0 => forward_direction,
            _ => {
                let prev_grid_position = body_query.get(ordered_entities[i - 1]).unwrap().1;
                grid_direction(&prev_grid_position, &grid_position)
            }
        };

        // determine the rotation and scene to show for the visual
        let (rotation, scene) = match forward_direction.abs() == back_direction.abs() {
            true => (
                match forward_direction {
                    Dir3::NEG_Z => 0.0,
                    Dir3::Z => PI,
                    Dir3::NEG_X => PI * 0.5,
                    _ => PI * 1.5,
                },
                match i {
                    0 => assets.body_end.clone(),
                    _ => assets.body_straight.clone(),
                },
            ),
            false => (
                if forward_direction == Dir3::NEG_Z && back_direction == Dir3::X
                    || forward_direction == Dir3::NEG_X && back_direction == Dir3::Z
                {
                    PI * 1.5
                } else if forward_direction == Dir3::Z && back_direction == Dir3::NEG_X
                    || forward_direction == Dir3::X && back_direction == Dir3::NEG_Z
                {
                    PI * 0.5
                } else if forward_direction == Dir3::NEG_Z && back_direction == Dir3::NEG_X
                    || forward_direction == Dir3::X && back_direction == Dir3::Z
                {
                    PI
                } else {
                    0.0
                },
                assets.body_corner.clone(),
            ),
        };

        // apply the rotation and scene
        let (_, _, _, mut transform, mut scene_root) = body_query.get_mut(entity).unwrap();
        transform.rotation = Quat::from_rotation_y(rotation);
        scene_root.0 = scene;
    }
}

fn grid_direction(first: &GridPosition, second: &GridPosition) -> Dir3 {
    if second.0.z < first.0.z {
        Dir3::NEG_Z
    } else if second.0.z > first.0.z {
        Dir3::Z
    } else if second.0.x < first.0.x {
        Dir3::NEG_X
    } else {
        Dir3::X
    }
}
