use bevy::{color::palettes::tailwind, prelude::*, render::camera::ScalingMode};
use rand::{thread_rng, Rng};

// - CONSTANTS -

const ARENA_SIZE: i32 = 11;
const ARENA_HALF_SIZE: i32 = ARENA_SIZE / 2;

// - ENTRY -

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<FoodEaten>()
        .add_event::<FoodNeeded>()
        .add_observer(construct_snake_visual)
        .add_observer(construct_food)
        .add_observer(construct_wall)
        .add_systems(
            PreStartup,
            (
                insert_snake_material,
                insert_food_material,
                insert_unit_cube_mesh,
                insert_food_mesh,
                insert_wall_material,
            ),
        )
        .add_systems(
            Startup,
            (
                spawn_camera,
                spawn_light,
                spawn_level_state,
                spawn_score_label,
                spawn_walls,
                spawn_snake,
                spawn_initial_food,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                move_snake,
                eat_food,
                (
                    (spawn_next_food, spawn_food).chain(),
                    (increment_score, update_score_label).chain(),
                ),
            )
                .chain(),
        )
        .add_systems(Update, control_snake)
        .add_systems(PostUpdate, apply_grid_position)
        .run();
}

// - EVENTS -

#[derive(Event)]
struct FoodEaten;

#[derive(Event)]
struct FoodNeeded;

// - RESOURCES -

#[derive(Resource)]
struct SnakeMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
struct FoodMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
struct UnitCubeMesh(Handle<Mesh>);

#[derive(Resource)]
struct FoodMesh(Handle<Mesh>);

#[derive(Resource)]
struct WallMaterial(Handle<StandardMaterial>);

// - SCENE COMPONENTS -

#[derive(Component)]
#[require(
    SnakeVisual,
    SnakeMoveTimer,
    SnakeNextDirection,
    SnakeDirection,
    SnakeBodyBuffer,
    GridPosition
)]
struct SnakeHead;

#[derive(Component)]
#[require(SnakeVisual)]
struct SnakeBodySegment;

#[derive(Component)]
struct Food;

#[derive(Component, Default)]
struct SnakeVisual;

#[derive(Component)]
struct Wall;

#[derive(Component)]
#[require(Score)]
struct LevelState;

#[derive(Component)]
#[require(Text(Self::text), Node(Self::node), TextFont(Self::text_font))]
struct ScoreLabel;

impl ScoreLabel {
    fn text() -> Text {
        Text::new("0")
    }

    fn node() -> Node {
        Node {
            margin: UiRect::all(Val::Px(16.)),
            ..default()
        }
    }

    fn text_font() -> TextFont {
        TextFont::from_font_size(48.)
    }
}

// - COMPONENTS -

#[derive(Component)]
struct SnakeMoveTimer(Timer);

impl Default for SnakeMoveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Repeating))
    }
}

#[derive(Component)]
struct SnakeNextDirection(Dir3);

impl Default for SnakeNextDirection {
    fn default() -> Self {
        Self(Dir3::NEG_Z)
    }
}

#[derive(Component)]
struct SnakeDirection(Dir3);

impl Default for SnakeDirection {
    fn default() -> Self {
        Self(SnakeNextDirection::default().0)
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

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
struct GridPosition(IVec3);

#[derive(Component, Default)]
struct Score(u32);

// - OBSERVERS -

fn construct_snake_visual(
    trigger: Trigger<OnAdd, SnakeVisual>,
    snake_material: Res<SnakeMaterial>,
    unit_cube_mesh: Res<UnitCubeMesh>,
    mut commands: Commands,
) {
    commands.entity(trigger.entity()).insert((
        Mesh3d(unit_cube_mesh.0.clone()),
        MeshMaterial3d(snake_material.0.clone()),
    ));
}

fn construct_food(
    trigger: Trigger<OnAdd, Food>,
    food_material: Res<FoodMaterial>,
    food_mesh: Res<FoodMesh>,
    mut commands: Commands,
) {
    commands.entity(trigger.entity()).insert((
        Mesh3d(food_mesh.0.clone()),
        MeshMaterial3d(food_material.0.clone()),
    ));
}

fn construct_wall(
    trigger: Trigger<OnAdd, Wall>,
    wall_material: Res<WallMaterial>,
    unit_cube_mesh: Res<UnitCubeMesh>,
    mut commands: Commands,
) {
    commands.entity(trigger.entity()).insert((
        Mesh3d(unit_cube_mesh.0.clone()),
        MeshMaterial3d(wall_material.0.clone()),
    ));
}

// - SYSTEMS -

fn insert_snake_material(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(SnakeMaterial(materials.add(StandardMaterial {
        base_color: Color::from(tailwind::GREEN_500),
        perceptual_roughness: 1.0,
        ..default()
    })));
}

fn insert_food_material(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(FoodMaterial(materials.add(StandardMaterial {
        base_color: Color::from(tailwind::RED_500),
        perceptual_roughness: 1.0,
        ..default()
    })));
}

fn insert_unit_cube_mesh(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    commands.insert_resource(UnitCubeMesh(meshes.add(Cuboid::from_length(1.0))));
}

fn insert_food_mesh(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    commands.insert_resource(FoodMesh(meshes.add(Sphere::new(0.5))));
}

fn insert_wall_material(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(WallMaterial(materials.add(StandardMaterial {
        base_color: Color::from(tailwind::SLATE_400),
        perceptual_roughness: 1.0,
        ..default()
    })));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 16.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.75, 2.0, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_level_state(mut commands: Commands) {
    commands.spawn(LevelState);
}

fn spawn_score_label(mut commands: Commands) {
    commands.spawn(ScoreLabel);
}

fn spawn_walls(mut commands: Commands) {
    // left
    commands.spawn((
        Wall,
        Transform::from_xyz(-ARENA_HALF_SIZE as f32 - 1.0, 0.0, 0.0).with_scale(Vec3::new(
            1.0,
            1.0,
            ARENA_SIZE as f32 + 2.0,
        )),
    ));

    // right
    commands.spawn((
        Wall,
        Transform::from_xyz(ARENA_HALF_SIZE as f32 + 1.0, 0.0, 0.0).with_scale(Vec3::new(
            1.0,
            1.0,
            ARENA_SIZE as f32 + 2.0,
        )),
    ));

    // top
    commands.spawn((
        Wall,
        Transform::from_xyz(0.0, 0.0, -ARENA_HALF_SIZE as f32 - 1.0).with_scale(Vec3::new(
            ARENA_SIZE as f32,
            1.0,
            1.0,
        )),
    ));

    // bottom
    commands.spawn((
        Wall,
        Transform::from_xyz(0.0, 0.0, ARENA_HALF_SIZE as f32 + 1.0).with_scale(Vec3::new(
            ARENA_SIZE as f32,
            1.0,
            1.0,
        )),
    ));
}

fn spawn_snake(mut commands: Commands) {
    commands.spawn(SnakeHead);
}

fn spawn_initial_food(mut food_needed: EventWriter<FoodNeeded>) {
    food_needed.send(FoodNeeded);
}

fn move_snake(
    mut head_query: Query<
        (
            &SnakeNextDirection,
            &mut SnakeMoveTimer,
            &mut GridPosition,
            &mut SnakeBodyBuffer,
            &mut SnakeDirection,
        ),
        Without<SnakeBodyIndex>,
    >,
    mut body_query: Query<(&mut SnakeBodyIndex, &mut GridPosition)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (next_direction, mut timer, mut grid_position, mut buffer, mut direction) in
        head_query.iter_mut()
    {
        if !timer.0.tick(time.delta()).just_finished() {
            continue;
        }

        // move the head forward by the snake's direction, detecting arena bounds collision
        let prev_position = grid_position.0;
        let next_position = grid_position.0 + next_direction.0.as_ivec3();

        // check the next position for a wall or any other snake part
        if [next_position.x, next_position.z]
            .iter()
            .any(|e| e > &ARENA_HALF_SIZE || e < &-ARENA_HALF_SIZE)
            || body_query.iter().any(|(_, gp)| gp.0 == next_position)
        {
            timer.0.pause();
            continue;
        }
        grid_position.0 += next_direction.0.as_ivec3();
        direction.0 = next_direction.0;

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

fn control_snake(
    mut query: Query<(&mut SnakeNextDirection, &SnakeDirection)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    // latching input respects only the final input, even if it was invalid for the current direction
    for (mut next_direction, direction) in query.iter_mut() {
        if input.just_pressed(KeyCode::KeyA) {
            next_direction.0 = match direction.0 {
                Dir3::X => direction.0,
                _ => Dir3::NEG_X,
            };
        } else if input.just_pressed(KeyCode::KeyD) {
            next_direction.0 = match direction.0 {
                Dir3::NEG_X => direction.0,
                _ => Dir3::X,
            };
        } else if input.just_pressed(KeyCode::KeyW) {
            next_direction.0 = match direction.0 {
                Dir3::Z => direction.0,
                _ => Dir3::NEG_Z,
            };
        } else if input.just_pressed(KeyCode::KeyS) && direction.0 != Dir3::NEG_Z {
            next_direction.0 = match direction.0 {
                Dir3::NEG_Z => direction.0,
                _ => Dir3::Z,
            };
        }
    }
}

fn eat_food(
    mut snake_query: Query<
        (&GridPosition, &mut SnakeBodyBuffer),
        (With<SnakeHead>, Changed<GridPosition>),
    >,
    food_query: Query<(Entity, &GridPosition), (With<Food>, Without<SnakeHead>)>,
    mut food_eaten: EventWriter<FoodEaten>,
    mut commands: Commands,
) {
    for (snake_grid_position, mut buffer) in snake_query.iter_mut() {
        for food_entity in food_query
            .iter()
            .filter_map(|(e, gp)| (gp == snake_grid_position).then(|| e))
        {
            commands.entity(food_entity).despawn_recursive();
            buffer.0 += 1; // extend the body
            food_eaten.send(FoodEaten);
        }
    }
}

fn spawn_next_food(
    mut food_eaten: EventReader<FoodEaten>,
    mut food_needed: EventWriter<FoodNeeded>,
) {
    for _ in food_eaten.read() {
        food_needed.send(FoodNeeded);
    }
}

fn spawn_food(
    mut food_needed: EventReader<FoodNeeded>,
    query: Query<&GridPosition>,
    mut commands: Commands,
) {
    if food_needed.is_empty() {
        return;
    }

    let mut pool = Vec::with_capacity(ARENA_SIZE as usize * ARENA_SIZE as usize);
    for x in -ARENA_HALF_SIZE..=ARENA_HALF_SIZE {
        for z in -ARENA_HALF_SIZE..=ARENA_HALF_SIZE {
            let grid_position = GridPosition(IVec3::new(x, 0, z));
            if query.iter().all(|gp| gp != &grid_position) {
                pool.push(grid_position);
            }
        }
    }

    let mut rng = thread_rng();

    for _ in food_needed.read() {
        let grid_position = pool.swap_remove(rng.gen_range(0..pool.len()));
        commands.spawn((Food, grid_position));
    }
}

fn apply_grid_position(mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation = grid_position.0.as_vec3();
    }
}

fn increment_score(mut food_eaten: EventReader<FoodEaten>, mut query: Query<&mut Score>) {
    for _ in food_eaten.read() {
        for mut score in query.iter_mut() {
            score.0 += 1;
        }
    }
}

fn update_score_label(
    score_query: Query<&Score, Changed<Score>>,
    mut label_query: Query<&mut Text, With<ScoreLabel>>,
) {
    for score in score_query.iter() {
        for mut text in label_query.iter_mut() {
            text.0 = score.0.to_string();
        }
    }
}
