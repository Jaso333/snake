use bevy::prelude::*;

use crate::{
    game::{GameEntity, SpawnLevel},
    level::Score,
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_level)
            .add_systems(FixedPostUpdate, update_score_label);
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
