mod arena;
mod food;
mod game;
mod grid;
mod hud;
mod level;
mod snake;

use arena::ArenaPlugin;
use bevy::prelude::*;
use food::FoodPlugin;
use game::GamePlugin;
use grid::GridPlugin;
use hud::HudPlugin;
use level::LevelPlugin;
use snake::SnakePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(HudPlugin)
        .add_plugins(ArenaPlugin)
        .add_plugins(SnakePlugin)
        .add_plugins(GridPlugin)
        .add_plugins(FoodPlugin)
        .run();
}
