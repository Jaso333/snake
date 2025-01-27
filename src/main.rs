use bevy::{color::palettes::tailwind, prelude::*, render::camera::ScalingMode};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_observer(construct_snake_visual)
        .add_systems(
            PreStartup,
            (insert_snake_material, insert_snake_segment_mesh),
        )
        .add_systems(Startup, (spawn_camera, spawn_light, spawn_snake))
        .add_systems(FixedUpdate, move_snake)
        .add_systems(Update, control_snake)
        .run();
}

#[derive(Resource)]
struct SnakeMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
struct SnakeSegmentMesh(Handle<Mesh>);

#[derive(Component)]
#[require(SnakeVisual, SnakeMoveTimer, SnakeDirection, SnakeBodyBuffer)]
struct SnakeHead;

#[derive(Component)]
#[require(SnakeVisual)]
struct SnakeBodySegment;

#[derive(Component, Default)]
struct SnakeVisual;

#[derive(Component)]
struct SnakeMoveTimer(Timer);

#[derive(Component)]
struct SnakeDirection(Dir3);

#[derive(Component)]
struct SnakeBodyBuffer(usize);

#[derive(Component, PartialEq, Eq, PartialOrd, Ord)]
struct SnakeBodyIndex(u32);

impl Default for SnakeBodyBuffer {
    fn default() -> Self {
        Self(3)
    }
}

impl Default for SnakeDirection {
    fn default() -> Self {
        Self(Dir3::NEG_Z)
    }
}

impl Default for SnakeMoveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Repeating))
    }
}

fn construct_snake_visual(
    trigger: Trigger<OnAdd, SnakeVisual>,
    snake_material: Res<SnakeMaterial>,
    snake_mesh: Res<SnakeSegmentMesh>,
    mut commands: Commands,
) {
    commands.entity(trigger.entity()).insert((
        Mesh3d(snake_mesh.0.clone()),
        MeshMaterial3d(snake_material.0.clone()),
    ));
}

fn insert_snake_material(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(SnakeMaterial(
        materials.add(Color::from(tailwind::GREEN_500)),
    ));
}

fn insert_snake_segment_mesh(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    commands.insert_resource(SnakeSegmentMesh(meshes.add(Cuboid::from_length(1.0))));
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

fn spawn_snake(mut commands: Commands) {
    commands.spawn(SnakeHead);
}

fn move_snake(
    mut head_query: Query<
        (
            &SnakeDirection,
            &mut SnakeMoveTimer,
            &mut Transform,
            &mut SnakeBodyBuffer,
        ),
        Without<SnakeBodyIndex>,
    >,
    mut body_query: Query<(&mut SnakeBodyIndex, &mut Transform)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (direction, mut timer, mut transform, mut buffer) in head_query.iter_mut() {
        if !timer.0.tick(time.delta()).just_finished() {
            continue;
        }

        // move the head forward by the snake's direction
        let prev_translation = transform.translation;
        transform.translation += direction.0.as_vec3();

        // set the defaults for the next body piece
        let mut next_body_index = 0;
        let mut next_body_translation = prev_translation;

        // get the current max index
        if let Some((max_index, _)) = body_query.iter().max_by(|(i1, ..), (i2, ..)| i1.cmp(i2)) {
            let max_index = max_index.0;

            // find the segment at the back of the snake
            if let Some((mut min_index, mut min_transform)) = body_query
                .iter_mut()
                .min_by(|(i1, ..), (i2, ..)| i1.cmp(i2))
            {
                // move the body segment from the back to the head's previous position
                next_body_translation = min_transform.translation;
                next_body_index = min_index.0;
                min_index.0 = max_index + 1;
                min_transform.translation = prev_translation;
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
            Transform::from_translation(next_body_translation),
        ));
    }
}

fn control_snake(mut query: Query<&mut SnakeDirection>, input: Res<ButtonInput<KeyCode>>) {
    for mut direction in query.iter_mut() {
        if input.just_pressed(KeyCode::KeyA) {
            direction.0 = Dir3::NEG_X;
        } else if input.just_pressed(KeyCode::KeyD) {
            direction.0 = Dir3::X;
        } else if input.just_pressed(KeyCode::KeyW) {
            direction.0 = Dir3::NEG_Z;
        } else if input.just_pressed(KeyCode::KeyS) {
            direction.0 = Dir3::Z;
        }
    }
}
