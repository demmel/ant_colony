use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    assets::{Colors, Meshes},
    config::LAYER_FOOD,
};

#[derive(Component)]
pub struct Food {
    pub amount: f32,
}

pub fn spawn_food(
    commands: &mut Commands,
    meshes: &Res<Meshes>,
    colors: &Res<Colors>,
    x: f32,
    y: f32,
    amount: f32,
) -> Entity {
    commands
        .spawn((
            Food { amount },
            MaterialMesh2dBundle {
                mesh: meshes.food.clone(),
                material: colors.food.clone(),
                transform: Transform::from_translation(Vec3::new(x, y, LAYER_FOOD))
                    .with_scale(Vec3::splat((amount / std::f32::consts::PI).sqrt())),
                ..Default::default()
            },
        ))
        .id()
}

pub fn update_food_size(mut food: Query<(&Food, &mut Transform), Changed<Food>>) {
    for (food, mut transform) in food.iter_mut() {
        transform.scale = Vec3::splat((food.amount / std::f32::consts::PI).sqrt());
    }
}
