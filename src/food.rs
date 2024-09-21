use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
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

pub fn spawn_food(commands: &mut Commands, x: f32, y: f32, amount: f32) -> Entity {
    commands
        .spawn((
            Food { amount },
            SpatialBundle::from_transform(Transform::from_translation(Vec3::new(x, y, 0.0))),
        ))
        .id()
}

pub fn spawn_random_food(commands: &mut Commands) {
    let mut rng = rand::thread_rng();
    let min_distance_from_edge = 60.0;
    let half_height = WORLD_HEIGHT / 2.0 - min_distance_from_edge;
    let half_width = WORLD_WIDTH / 2.0 - min_distance_from_edge;
    let x = rng.gen_range(-half_width..half_width);
    let y = rng.gen_range(-half_height..half_height);
    let amount = rng.gen_range(50.0..250.0);
    spawn_food(commands, x, y, amount);
}

pub fn setup_food_rendering(
    mut commands: Commands,
    meshes: Res<Meshes>,
    colors: Res<Colors>,
    food: Query<(Entity, &Food), Added<Food>>,
) {
    for (entity, food) in food.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((MaterialMesh2dBundle {
                mesh: meshes.food.clone(),
                material: colors.food.clone(),
                transform: Transform::from_translation(Vec3::Z * LAYER_FOOD)
                    .with_scale(Vec3::splat((food.amount / std::f32::consts::PI).sqrt())),
                ..Default::default()
            },));
        });
    }
}

pub fn update_food_size(
    food: Query<(&Food, &Children), Changed<Food>>,
    mut food_meshes: Query<&mut Transform, With<Mesh2dHandle>>,
) {
    for (food, children) in food.iter() {
        let radius = (food.amount / std::f32::consts::PI).sqrt();
        for child in children.iter() {
            if let Ok(mut food_mesh_transform) = food_meshes.get_mut(*child) {
                food_mesh_transform.scale = Vec3::splat(radius);
            }
        }
    }
}
