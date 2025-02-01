mod arena;
mod food;
mod game;
mod game_over;
mod grid;
mod hud;
mod level;
mod snake;

use arena::ArenaPlugin;
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use food::FoodPlugin;
use game::GamePlugin;
use game_over::GameOverPlugin;
use grid::GridPlugin;
use hud::HudPlugin;
use level::LevelPlugin;
use snake::SnakePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TweeningPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(HudPlugin)
        .add_plugins(ArenaPlugin)
        .add_plugins(SnakePlugin)
        .add_plugins(GridPlugin)
        .add_plugins(FoodPlugin)
        .add_plugins(GameOverPlugin)
        .run();
}
