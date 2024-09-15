use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    assets::{Colors, Meshes},
    config::LAYER_NEST,
};

#[derive(Component)]
pub struct Nest {
    pub food: f32,
}

pub fn spawn_nest(
    commands: &mut Commands,
    meshes: &Res<Meshes>,
    colors: &Res<Colors>,
    x: f32,
    y: f32,
) {
    commands.spawn((
        Nest { food: 0.0 },
        MaterialMesh2dBundle {
            mesh: meshes.nest.clone(),
            material: colors.nest.clone(),
            transform: Transform::from_translation(Vec3::new(x, y, LAYER_NEST)),
            ..Default::default()
        },
    ));
}
