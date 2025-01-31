use bevy::prelude::*;

use crate::game::{GameEntity, SpawnLevel};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_level);
    }
}

#[derive(Component)]
#[require(
    GameEntity,
    Text(Self::text),
    Node(Self::node),
    TextFont(Self::text_font)
)]
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

fn on_spawn_level(_: Trigger<SpawnLevel>, mut commands: Commands) {
    commands.spawn(ScoreLabel);
}
