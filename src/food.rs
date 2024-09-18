use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::prelude::*;

use crate::{
    assets::{Colors, Meshes},
    config::{LAYER_FOOD, WORLD_HEIGHT, WORLD_WIDTH},
};

#[derive(Component)]
pub struct Food {
    amount: f32,
}

impl Food {
    pub fn empty(&self) -> bool {
        self.amount <= 0.0
    }

    pub fn radius(&self) -> f32 {
        (self.amount / std::f32::consts::PI).sqrt()
    }

    pub fn amount(&self) -> f32 {
        self.amount
    }

    pub fn remove(&mut self, amount: f32) -> f32 {
        let amount = amount.min(self.amount);
        self.amount -= amount;
        amount
    }
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

pub fn spawn_random_food(commands: &mut Commands, meshes: &Res<Meshes>, colors: &Res<Colors>) {
    let mut rng = rand::thread_rng();
    let min_distance_from_edge = 60.0;
    let half_height = WORLD_HEIGHT / 2.0 - min_distance_from_edge;
    let half_width = WORLD_WIDTH / 2.0 - min_distance_from_edge;
    let x = rng.gen_range(-half_width..half_width);
    let y = rng.gen_range(-half_height..half_height);
    let amount = rng.gen_range(50.0..250.0);
    spawn_food(commands, &meshes, &colors, x, y, amount);
}

pub fn update_food_size(mut food: Query<(&Food, &mut Transform), Changed<Food>>) {
    for (food, mut transform) in food.iter_mut() {
        transform.scale = Vec3::splat((food.amount / std::f32::consts::PI).sqrt());
    }
}
