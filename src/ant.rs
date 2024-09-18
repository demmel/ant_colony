use bevy::{math::NormedVectorSpace, prelude::*, sprite::MaterialMesh2dBundle};
use rand::prelude::*;

use crate::{
    assets::{Colors, Meshes},
    config::*,
    food::{spawn_random_food, Food},
    nest::Nest,
    position_index::TrackPositionIndex,
    track::{spawn_track, Track, TrackKind},
};

#[derive(Component)]
pub struct Ant;

#[derive(Component)]
pub struct Satiation(f32);

impl Satiation {
    pub fn amount(&self) -> f32 {
        self.0
    }

    pub fn add(&mut self, amount: f32) -> f32 {
        let added = (ANT_MAX_ENERGY - self.0).min(amount);
        self.0 += added;
        if (ANT_MAX_ENERGY - self.0) < 0.0001 {
            self.0 = ANT_MAX_ENERGY;
        }
        added
    }

    pub fn remove(&mut self, amount: f32) -> f32 {
        let removed = self.0.min(amount);
        self.0 -= removed;
        if self.0 < 0.0001 {
            self.0 = 0.0;
        }
        removed
    }

    pub fn empty(&self) -> bool {
        self.0 <= 0.0
    }
}

#[derive(Component)]
pub struct HeldFood(f32);

impl HeldFood {
    pub fn empty(&self) -> bool {
        self.0 <= 0.0
    }

    pub fn full(&self) -> bool {
        self.0 >= ANT_MAX_CARRY
    }

    pub fn amount(&self) -> f32 {
        self.0
    }

    pub fn add(&mut self, amount: f32) -> f32 {
        let added = (ANT_MAX_CARRY - self.0).min(amount);
        self.0 += added;
        if (ANT_MAX_CARRY - self.0) < 0.0001 {
            self.0 = ANT_MAX_CARRY;
        }
        added
    }

    pub fn remove(&mut self, amount: f32) -> f32 {
        let removed = self.0.min(amount);
        self.0 -= removed;
        if self.0 < 0.0001 {
            self.0 = 0.0;
        }
        removed
    }
}

#[derive(Component)]
pub struct CarriedFood;

pub fn spawn_ant(
    commands: &mut Commands,
    meshes: &Res<Meshes>,
    colors: &Res<Colors>,
    x: f32,
    y: f32,
    rotation: f32,
) -> Entity {
    commands
        .spawn((
            Ant,
            Satiation(1.0),
            HeldFood(0.0),
            SpatialBundle::from_transform(
                Transform::from_translation(Vec3::new(x, y, LAYER_ANT))
                    .mul_transform(Transform::from_rotation(Quat::from_rotation_z(rotation))),
            ),
        ))
        .with_children(|parent| {
            let head_y = 3.0 * ANT_SEGMENT_RADIUS / 2.0;
            let antenna_y = head_y + ANT_SEGMENT_RADIUS;
            let antenna_x = ANT_SEGMENT_RADIUS;

            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_segment.clone(),
                material: colors.ant.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, head_y, 0.0)),
                ..Default::default()
            });
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_segment.clone(),
                material: colors.ant.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                ..Default::default()
            });
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_segment.clone(),
                material: colors.ant.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, -head_y, 0.0)),
                ..Default::default()
            });
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_antenna.clone(),
                material: colors.ant.clone(),
                transform: Transform::from_translation(Vec3::new(antenna_x, antenna_y, 0.0)),
                ..Default::default()
            });
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_antenna.clone(),
                material: colors.ant.clone(),
                transform: Transform::from_translation(Vec3::new(-antenna_x, antenna_y, 0.0)),
                ..Default::default()
            });
            parent.spawn((
                CarriedFood,
                MaterialMesh2dBundle {
                    mesh: meshes.food.clone(),
                    material: colors.food.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        head_y + ANT_SEGMENT_RADIUS,
                        0.1,
                    )),
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
            ));
        })
        .id()
}

pub fn decay_satiation(mut satiations: Query<&mut Satiation>) {
    for mut satiation in satiations.iter_mut() {
        satiation.remove(ANT_ENERGY_LOSS_RATE * FIXED_DELTA_TIME);
    }
}

pub fn eat_held_food(mut eaters: Query<(&mut HeldFood, &mut Satiation)>) {
    for (mut held_food, mut satiation) in eaters.iter_mut() {
        let eats = held_food.remove((ANT_MAX_ENERGY - satiation.amount()) * 0.1 * FIXED_DELTA_TIME);
        satiation.add(eats);
    }
}

pub fn starve(mut commands: Commands, satiations: Query<(Entity, &Satiation)>) {
    for (entity, satiation) in satiations.iter() {
        if satiation.empty() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn walk_ants(mut ants: Query<&mut Transform, With<Ant>>) {
    let min_distance_from_edge = ANT_SEGMENT_RADIUS * 2.0 * 1.5;

    for mut transform in ants.iter_mut() {
        let forward = transform.up();
        transform.translation += forward * ANT_SPEED * FIXED_DELTA_TIME;

        if transform.translation.x < -WORLD_WIDTH / 2.0 + min_distance_from_edge {
            transform.translation.x = -WORLD_WIDTH / 2.0 + min_distance_from_edge;
        } else if transform.translation.x > WORLD_WIDTH / 2.0 - min_distance_from_edge {
            transform.translation.x = WORLD_WIDTH / 2.0 - min_distance_from_edge;
        }

        if transform.translation.y < -WORLD_HEIGHT / 2.0 + min_distance_from_edge {
            transform.translation.y = -WORLD_HEIGHT / 2.0 + min_distance_from_edge;
        } else if transform.translation.y > WORLD_HEIGHT / 2.0 - min_distance_from_edge {
            transform.translation.y = WORLD_HEIGHT / 2.0 - min_distance_from_edge;
        }
    }
}

pub fn rotate_ants(
    track_position_index: Res<TrackPositionIndex>,
    mut ants: Query<(&mut Transform, &HeldFood), With<Ant>>,
    tracks: Query<&Track>,
    food: Query<(Entity, &Food, &Transform), Without<Ant>>,
    nests: Query<&Transform, (With<Nest>, Without<Ant>)>,
) {
    let mut rng = rand::thread_rng();
    let nest_transform = nests.single();

    let sense_offsets = [
        Vec3::Y * ANT_SENSE_DISTANCE,
        Quat::from_rotation_z(std::f32::consts::PI / 8.0).mul_vec3(Vec3::Y) * ANT_SENSE_DISTANCE,
        Quat::from_rotation_z(-std::f32::consts::PI / 8.0).mul_vec3(Vec3::Y) * ANT_SENSE_DISTANCE,
    ];

    for (mut ant_transform, held_food) in ants.iter_mut() {
        let forward = ant_transform.up();

        let mut direction: Vec2 = sense_offsets
            .iter()
            .map(|sense_offset| {
                let sense_offset = ant_transform.rotation.mul_vec3(*sense_offset).xy();
                let sense_center = ant_transform.translation.xy() + sense_offset;

                let sensed_tracks =
                    track_position_index.within(sense_center, ANT_SENSE_RADIUS + TRACK_RADIUS);

                let sensed_food = food.iter().find(|(_, food, food_transform)| {
                    food_transform.translation.xy().distance(sense_center)
                        < ANT_SENSE_RADIUS + food.radius()
                });

                let sensed_nest = nest_transform.translation.xy().distance(sense_center)
                    < ANT_SENSE_RADIUS + NEST_RADIUS;

                let weight = if !held_food.empty() && sensed_nest {
                    10.0f32
                } else if held_food.empty() && sensed_nest {
                    0.0
                } else if let Some((_, _, _)) = sensed_food {
                    if !held_food.empty() {
                        0.0
                    } else {
                        10.0
                    }
                } else {
                    rng.gen_range(0.0..0.1)
                        + sensed_tracks
                            .map(|(_, id)| {
                                let track = tracks.get(*id).unwrap();

                                let multiplier = match (&track.kind, !held_food.empty()) {
                                    (TrackKind::Nest, false) => 0.0,
                                    (TrackKind::Nest, true) => 1.0,
                                    (TrackKind::Food, false) => 1.0,
                                    (TrackKind::Food, true) => 0.0,
                                };

                                track.concentration * multiplier
                            })
                            .sum::<f32>()
                };

                weight.max(0.01) * (sense_center - ant_transform.translation.xy())
            })
            .sum::<Vec2>()
            .normalize();

        let min_distance_fron_edge = 10.0;
        let soft_min_distance_from_edge = 50.0;
        let distance = ant_transform.translation.x.distance(WORLD_WIDTH / 2.0);
        if distance < soft_min_distance_from_edge {
            direction.x -= 2.0
                * (1.0
                    - (distance - min_distance_fron_edge)
                        / (soft_min_distance_from_edge - min_distance_fron_edge));
        }

        let distance = ant_transform.translation.x.distance(-WORLD_WIDTH / 2.0);
        if distance < soft_min_distance_from_edge {
            direction.x += 2.0
                * (1.0
                    - (distance - min_distance_fron_edge)
                        / (soft_min_distance_from_edge - min_distance_fron_edge));
        }

        let distance = ant_transform.translation.y.distance(WORLD_HEIGHT / 2.0);
        if distance < soft_min_distance_from_edge {
            direction.y -= 2.0
                * (1.0
                    - (distance - min_distance_fron_edge)
                        / (soft_min_distance_from_edge - min_distance_fron_edge));
        }

        let distance = ant_transform.translation.y.distance(-WORLD_HEIGHT / 2.0);
        if distance < soft_min_distance_from_edge {
            direction.y += 2.0
                * (1.0
                    - (distance - min_distance_fron_edge)
                        / (soft_min_distance_from_edge - min_distance_fron_edge));
        }

        let angle = if direction != Vec2::ZERO {
            direction = direction.normalize();
            let angle = forward.xy().angle_between(direction);
            angle + rng.gen_range(-std::f32::consts::PI / 8.0..std::f32::consts::PI / 8.0)
        } else {
            rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI)
        };

        let max_turn = ANT_ROTATION_SPEED * FIXED_DELTA_TIME;
        let angle = angle.clamp(-max_turn, max_turn);

        ant_transform.rotation *= Quat::from_rotation_z(angle);
    }
}

pub fn emit_pheromones(
    mut commands: Commands,
    meshes: Res<Meshes>,
    colors: Res<Colors>,
    track_position_index: Res<TrackPositionIndex>,
    ants: Query<(&Transform, &HeldFood), With<Ant>>,
    mut tracks: Query<(&mut Track, &Transform), Without<Ant>>,
) {
    for (ant_transform, held_food) in ants.iter() {
        let nearby_tracks =
            track_position_index.within(ant_transform.translation.xy(), TRACK_RADIUS * 2.0);

        let nearest_track = nearby_tracks.clone().find(|(distance, id)| {
            let (track, _) = tracks.get(**id).unwrap();
            match (&track.kind, !held_food.empty()) {
                (TrackKind::Nest, false) => {}
                (TrackKind::Nest, true) => {
                    return false;
                }
                (TrackKind::Food, false) => {
                    return false;
                }
                (TrackKind::Food, true) => {}
            };
            *distance < TRACK_RADIUS * 2.0
        });

        if let Some((_, id)) = nearest_track {
            let (mut track, _) = tracks.get_mut(*id).unwrap();
            track.concentration += ANT_TRACK_CONCENTRATION * FIXED_DELTA_TIME;
            track.concentration = track.concentration.min(1.0);
        } else {
            spawn_track(
                &mut commands,
                &meshes,
                &colors,
                ant_transform.translation.xy(),
                ANT_TRACK_CONCENTRATION * FIXED_DELTA_TIME,
                if !held_food.empty() {
                    TrackKind::Food
                } else {
                    TrackKind::Nest
                },
            );
        }
    }
}

pub fn pick_up_food(
    mut commands: Commands,
    meshes: Res<Meshes>,
    colors: Res<Colors>,
    mut ants: Query<(&Transform, &mut HeldFood), With<Ant>>,
    mut food: Query<(Entity, &mut Food, &Transform), Without<Ant>>,
) {
    for (ant_transform, mut held_food) in ants.iter_mut() {
        if held_food.full() {
            continue;
        }

        let nearby_food = food.iter_mut().find(|(_, _, transform)| {
            transform
                .translation
                .xy()
                .distance(ant_transform.translation.xy())
                < ANT_SENSE_RADIUS
        });

        if let Some((entity, mut food, food_transform)) = nearby_food {
            if food_transform
                .translation
                .xy()
                .distance(ant_transform.translation.xy())
                < ANT_SEGMENT_RADIUS * 1.5 + food.radius()
            {
                let took = held_food.add(food.amount());
                food.remove(took);
                if food.empty() {
                    commands.entity(entity).despawn();
                    spawn_random_food(&mut commands, &meshes, &colors);
                }
            }
        }
    }
}

pub fn deposit_food(
    mut ants: Query<(&Transform, &mut HeldFood), With<Ant>>,
    mut nests: Query<(&mut Nest, &Transform), Without<Ant>>,
) {
    let (mut nest, nest_transform) = nests.single_mut();

    for (ant_transform, mut held_food) in ants.iter_mut() {
        if held_food.empty() {
            continue;
        }

        let nest_distance = ant_transform
            .translation
            .xy()
            .distance(nest_transform.translation.xy());

        if nest_distance < ANT_SEGMENT_RADIUS * 1.5 + NEST_RADIUS {
            let amount = held_food.amount();
            nest.food += amount;
            held_food.remove(amount);
        }
    }
}

pub fn update_ant_holding_food(
    held_food_query: Query<(&HeldFood, &Children), Changed<HeldFood>>,
    mut carried_food_query: Query<(Entity, &CarriedFood, &mut Visibility, &mut Transform)>,
) {
    for (held_food, children) in held_food_query.iter() {
        for child in children.iter() {
            if let Ok((_, _, mut visibility, mut transform)) = carried_food_query.get_mut(*child) {
                *visibility = if !held_food.empty() {
                    transform.scale =
                        Vec3::splat((held_food.amount() / std::f32::consts::PI).sqrt());
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}
