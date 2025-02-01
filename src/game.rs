use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::Load)
                    .continue_to_state(GameState::Play)
                    .load_collection::<GameFont>(),
            )
            .add_observer(on_add_text_font)
            .add_systems(PreStartup, insert_unit_cube_mesh)
            .add_systems(OnEnter(GameState::Play), spawn_level);
    }
}

#[derive(Event)]
pub struct SpawnLevel;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
    #[default]
    Load,
    Play,
}

#[derive(Resource)]
pub struct UnitCubeMesh(pub Handle<Mesh>);

#[derive(Resource, AssetCollection)]
struct GameFont {
    #[asset(path = "font.ttf")]
    value: Handle<Font>,
}

#[derive(Component, Default)]
pub struct GameEntity;

fn on_add_text_font(
    trigger: Trigger<OnAdd, TextFont>,
    mut query: Query<&mut TextFont>,
    game_font: Res<GameFont>,
) {
    query.get_mut(trigger.entity()).unwrap().font = game_font.value.clone();
}

fn insert_unit_cube_mesh(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    commands.insert_resource(UnitCubeMesh(meshes.add(Cuboid::from_length(1.0))));
}

fn spawn_level(mut commands: Commands) {
    commands.trigger(SpawnLevel);
}
