use bevy::prelude::*;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, apply_grid_position.in_set(GridSet));
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct GridSet;

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
pub struct GridPosition(pub IVec3);

fn apply_grid_position(mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation = grid_position.0.as_vec3();
    }
}
