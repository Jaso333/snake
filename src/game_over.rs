use bevy::prelude::*;

use crate::{
    game::{DespawnGameEntities, GameEntity, SpawnLevel},
    snake::SnakeCollided,
};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_snake_collided);
    }
}

#[derive(Component)]
#[require(GameEntity, Node(Self::node))]
struct GameOverUi;

impl GameOverUi {
    fn node() -> Node {
        Node {
            display: Display::Grid,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            justify_items: JustifyItems::Center,
            row_gap: Val::Px(5.),
            ..default()
        }
    }
}

#[derive(Component)]
#[require(Text(Self::text), TextFont(Self::text_font))]
struct Title;

impl Title {
    fn text() -> Text {
        Text::new("Game Over")
    }

    fn text_font() -> TextFont {
        TextFont::from_font_size(64.)
    }
}

#[derive(Component)]
#[require(Button, Node(Self::node), BackgroundColor(Self::background_color))]
struct RestartButton;

impl RestartButton {
    fn node() -> Node {
        Node {
            padding: UiRect::all(Val::Px(10.)),
            ..default()
        }
    }

    fn background_color() -> BackgroundColor {
        BackgroundColor(Color::WHITE.with_alpha(0.5))
    }
}

#[derive(Component)]
#[require(Text(Self::text))]
struct RestartButtonText;

impl RestartButtonText {
    fn text() -> Text {
        Text::new("Restart")
    }
}

fn on_snake_collided(_: Trigger<SnakeCollided>, mut commands: Commands) {
    // spawn the game-over UI
    commands.spawn(GameOverUi).with_children(|cb| {
        cb.spawn(Title);
        cb.spawn(RestartButton)
            .observe(on_restart_button_click)
            .with_child(RestartButtonText);
    });
}

fn on_restart_button_click(mut trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    trigger.propagate(false);
    commands.trigger(DespawnGameEntities);
    commands.trigger(SpawnLevel);
}
