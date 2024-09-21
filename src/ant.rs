use bevy::{math::NormedVectorSpace, prelude::*, sprite::MaterialMesh2dBundle};
use enum_ordinalize::Ordinalize;
use rand::prelude::*;

use crate::{
    assets::{Colors, Meshes},
    config::*,
    food::{spawn_random_food, Food},
    nest::Nest,
    track::Tracks,
};

#[derive(Component)]
pub struct Ant;

#[derive(Component, Ordinalize, Clone, Copy)]
pub enum AntKind {
    Scout,
    Worker,
}

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
pub struct HeldFood {
    amount: f32,
    max: f32,
}

impl HeldFood {
    pub fn empty(&self) -> bool {
        self.amount <= 0.0
    }

    pub fn full(&self) -> bool {
        self.amount >= self.max
    }

    pub fn amount(&self) -> f32 {
        self.amount
    }

    pub fn add(&mut self, amount: f32) -> f32 {
        let added = (self.max - self.amount).min(amount);
        self.amount += added;
        if (self.max - self.amount) < 0.0001 {
            self.amount = self.max;
        }
        added
    }

    pub fn remove(&mut self, amount: f32) -> f32 {
        let removed = self.amount.min(amount);
        self.amount -= removed;
        if self.amount < 0.0001 {
            self.amount = 0.0;
        }
        removed
    }
}

#[derive(Component)]
pub struct CarriedFood;

pub fn spawn_ant(
    commands: &mut Commands,
    simulation_config: &Res<SimulationConfig>,
    x: f32,
    y: f32,
    rotation: f32,
    kind: AntKind,
) -> Entity {
    commands
        .spawn((
            Ant,
            Satiation(1.0),
            HeldFood {
                amount: 0.0,
                max: simulation_config.ant_max_carry,
            },
            SpatialBundle::from_transform(
                Transform::from_translation(Vec3::new(x, y, 0.0))
                    .mul_transform(Transform::from_rotation(Quat::from_rotation_z(rotation))),
            ),
            kind,
        ))
        .id()
}

pub fn setup_ant_rendering(
    mut commands: Commands,
    meshes: Res<Meshes>,
    colors: Res<Colors>,
    new_ants: Query<(Entity, &AntKind), Added<Ant>>,
) {
    for (entity, kind) in new_ants.iter() {
        let color = match kind {
            AntKind::Scout => colors.ant_scout.clone(),
            AntKind::Worker => colors.ant_worker.clone(),
        };
        commands.entity(entity).with_children(|parent| {
            let head_y = 3.0 * ANT_SEGMENT_RADIUS / 2.0;
            let antenna_y = head_y + ANT_SEGMENT_RADIUS;
            let antenna_x = ANT_SEGMENT_RADIUS;

            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_segment.clone(),
                material: color.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, head_y, LAYER_ANT)),
                ..Default::default()
            });
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_segment.clone(),
                material: color.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, LAYER_ANT)),
                ..Default::default()
            });
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_segment.clone(),
                material: color.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, -head_y, LAYER_ANT)),
                ..Default::default()
            });
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_antenna.clone(),
                material: color.clone(),
                transform: Transform::from_translation(Vec3::new(antenna_x, antenna_y, LAYER_ANT)),
                ..Default::default()
            });
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.ant_antenna.clone(),
                material: color.clone(),
                transform: Transform::from_translation(Vec3::new(-antenna_x, antenna_y, LAYER_ANT)),
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
                        LAYER_ANT + 0.1,
                    )),
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
            ));
        });
    }
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

enum AntGoal {
    Scout,
    Food,
    Nest,
}

pub fn rotate_ants(
    simulation_config: Res<SimulationConfig>,
    mut ants: Query<(&mut Transform, &Satiation, &HeldFood, &AntKind), With<Ant>>,
    tracks: Query<&Tracks>,
    food: Query<(Entity, &Food, &Transform), Without<Ant>>,
    nests: Query<&Transform, (With<Nest>, Without<Ant>)>,
) {
    let mut rng = rand::thread_rng();
    let nest_transform = nests.single();
    let tracks = tracks.single();
    let ant_sense_distance = simulation_config.ant_sense_distance;
    let ant_sense_radius = simulation_config.ant_sense_radius;

    let sense_offsets = [
        Vec3::Y * ant_sense_distance,
        Quat::from_rotation_z(3.0 * std::f32::consts::PI / 16.0).mul_vec3(Vec3::Y)
            * ant_sense_distance,
        Quat::from_rotation_z(-3.0 * std::f32::consts::PI / 16.0).mul_vec3(Vec3::Y)
            * ant_sense_distance,
    ];

    for (mut ant_transform, satiation, held_food, ant_kind) in ants.iter_mut() {
        let forward = ant_transform.up();

        let goal = if satiation.amount() < ANT_MAX_ENERGY * 0.5 {
            AntGoal::Nest
        } else {
            match ant_kind {
                AntKind::Scout => AntGoal::Scout,
                AntKind::Worker => {
                    if held_food.empty() {
                        AntGoal::Food
                    } else {
                        AntGoal::Nest
                    }
                }
            }
        };

        let mut direction: Vec2 = sense_offsets
            .iter()
            .map(|sense_offset| {
                let sense_offset = ant_transform.rotation.mul_vec3(*sense_offset).xy();
                let sense_center = ant_transform.translation.xy() + sense_offset;

                // gizmos.circle_2d(sense_center, ANT_SENSE_RADIUS, WHITE);

                let weight = match goal {
                    AntGoal::Food => {
                        let sensed_food = food.iter().find(|(_, food, food_transform)| {
                            food_transform.translation.xy().distance(sense_center)
                                < ant_sense_radius + food.radius()
                        });
                        if sensed_food.is_some() {
                            10.0
                        } else {
                            tracks
                                .within_circle(sense_center, ant_sense_radius)
                                .map(|track| track.food)
                                .sum::<f32>()
                        }
                    }
                    AntGoal::Nest => {
                        let sensed_nest = nest_transform.translation.xy().distance(sense_center)
                            < ant_sense_radius + NEST_RADIUS;
                        if sensed_nest {
                            10.0
                        } else {
                            tracks
                                .within_circle(sense_center, ant_sense_radius)
                                .map(|track| track.nest)
                                .sum::<f32>()
                        }
                    }
                    AntGoal::Scout => {
                        // Scout ants are attracted to low pheromone concentrations
                        1.0 - tracks
                            .within_circle(sense_center, ant_sense_radius)
                            .map(|track| track.food.max(track.nest))
                            .sum::<f32>()
                    }
                };

                weight.max(0.000001) * (sense_center - ant_transform.translation.xy())
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
            angle
                + rng.gen_range(
                    -3.0 * std::f32::consts::PI / 16.0..3.0 * std::f32::consts::PI / 16.0,
                )
        } else {
            rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI)
        };

        let max_turn = ANT_ROTATION_SPEED * FIXED_DELTA_TIME;
        let angle = angle.clamp(-max_turn, max_turn);

        // We need to normalize the rotation quaternion because it can drift over time due to floating point errors
        ant_transform.rotation =
            (Quat::from_rotation_z(angle) * ant_transform.rotation).normalize();
    }
}

pub fn emit_ant_pheromones(
    simulation_config: Res<SimulationConfig>,
    ants: Query<(&Transform, &HeldFood), With<Ant>>,
    mut tracks: Query<&mut Tracks>,
) {
    let mut tracks = tracks.single_mut();
    for (ant_transform, held_food) in ants.iter() {
        tracks.within_circle_mut(ant_transform.translation.xy(), TRACK_RADIUS, |track| {
            if !held_food.empty() {
                track.food += simulation_config.ant_track_concentration * FIXED_DELTA_TIME;
                track.food = track.food.min(1.0);
            } else {
                track.nest += simulation_config.ant_track_concentration * FIXED_DELTA_TIME;
                track.nest = track.nest.min(1.0);
            }
        });
    }
}

pub fn pick_up_food(
    mut commands: Commands,
    mut ants: Query<(&Transform, &mut HeldFood), With<Ant>>,
    mut food: Query<(Entity, &mut Food, &Transform), Without<Ant>>,
) {
    for (ant_transform, mut held_food) in ants.iter_mut() {
        if held_food.full() {
            continue;
        }

        let nearby_food = food.iter_mut().find(|(_, food, transform)| {
            transform
                .translation
                .xy()
                .distance(ant_transform.translation.xy())
                < ANT_SEGMENT_RADIUS * 1.5 + food.radius()
        });

        if let Some((entity, mut food, _)) = nearby_food {
            let took = held_food.add(food.amount());
            food.remove(took);
            if food.empty() {
                commands.entity(entity).despawn();
                spawn_random_food(&mut commands);
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

pub fn eat_nest_food(
    mut nests: Query<(&mut Nest, &Transform)>,
    mut satiations: Query<(&mut Satiation, &Transform)>,
) {
    for (mut nest, nest_transform) in nests.iter_mut() {
        for (mut satiation, satiation_transform) in satiations.iter_mut() {
            let distance = satiation_transform
                .translation
                .xy()
                .distance(nest_transform.translation.xy());
            if distance < ANT_SEGMENT_RADIUS * 1.5 + NEST_RADIUS {
                let eats = nest.food.min(ANT_MAX_ENERGY - satiation.amount());
                nest.food -= eats;
                satiation.add(eats);
            }
        }
    }
}

pub fn update_ant_holding_food(
    held_food_query: Query<(&HeldFood, &Children), Changed<HeldFood>>,
    mut carried_food_query: Query<(Entity, &CarriedFood, &mut Visibility)>,
) {
    for (held_food, children) in held_food_query.iter() {
        for child in children.iter() {
            if let Ok((_, _, mut visibility)) = carried_food_query.get_mut(*child) {
                *visibility = if !held_food.empty() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}
