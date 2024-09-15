use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use kdtree::{distance::squared_euclidean, KdTree};
use rand::prelude::*;

use crate::{
    assets::{Colors, Meshes},
    config::*,
    food::{spawn_food, Food},
    nest::Nest,
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
        let added = (1.0 - self.0).min(amount);
        self.0 += added;
        if (1.0 - self.0) < 0.0001 {
            self.0 = 1.0;
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

    pub fn amount(&self) -> f32 {
        self.0
    }

    pub fn add(&mut self, amount: f32) -> f32 {
        let added = (1.0 - self.0).min(amount);
        self.0 += added;
        if (1.0 - self.0) < 0.0001 {
            self.0 = 1.0;
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

pub fn decay_satiation(time: Res<Time>, mut satiations: Query<&mut Satiation>) {
    for mut satiation in satiations.iter_mut() {
        satiation.remove(0.01 * time.delta_seconds());
    }
}

pub fn eat_held_food(time: Res<Time>, mut eaters: Query<(&mut HeldFood, &mut Satiation)>) {
    for (mut held_food, mut satiation) in eaters.iter_mut() {
        let eats = held_food.remove((1.0 - satiation.amount()) * 0.1 * time.delta_seconds());
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

pub fn update_ants(
    mut commands: Commands,
    time: Res<Time>,
    meshes: Res<Meshes>,
    colors: Res<Colors>,
    mut ants: Query<(&mut Transform, &mut HeldFood), With<Ant>>,
    mut tracks: Query<(Entity, &mut Track, &Transform), Without<Ant>>,
    mut food: Query<(Entity, &mut Food, &Transform), (Without<Ant>, Without<Track>)>,
    mut nests: Query<
        (&mut Nest, &Transform),
        (With<Nest>, Without<Ant>, Without<Track>, Without<Food>),
    >,
) {
    let mut rng = rand::thread_rng();

    let (mut nest, nest_transform) = nests.single_mut();

    let mut track_lookup = KdTree::new(2);
    for (id, _, transform) in tracks.iter() {
        track_lookup
            .add([transform.translation.x, transform.translation.y], id)
            .unwrap();
    }

    for (mut ant_transform, mut held_food) in ants.iter_mut() {
        let forward = ant_transform.up();
        ant_transform.translation += forward * ANT_SPEED * time.delta_seconds();

        let min_distance_from_edge = ANT_SEGMENT_RADIUS * 2.0 * 1.5;

        if ant_transform.translation.x < -WORLD_WIDTH / 2.0 + min_distance_from_edge {
            ant_transform.translation.x = -WORLD_WIDTH / 2.0 + min_distance_from_edge;
        } else if ant_transform.translation.x > WORLD_WIDTH / 2.0 - min_distance_from_edge {
            ant_transform.translation.x = WORLD_WIDTH / 2.0 - min_distance_from_edge;
        }

        if ant_transform.translation.y < -WORLD_HEIGHT / 2.0 + min_distance_from_edge {
            ant_transform.translation.y = -WORLD_HEIGHT / 2.0 + min_distance_from_edge;
        } else if ant_transform.translation.y > WORLD_HEIGHT / 2.0 - min_distance_from_edge {
            ant_transform.translation.y = WORLD_HEIGHT / 2.0 - min_distance_from_edge;
        }

        let nearby_tracks = track_lookup
            .within(
                &[ant_transform.translation.x, ant_transform.translation.y],
                ANT_SENSE_RADIUS.powf(2.0),
                &squared_euclidean,
            )
            .unwrap();

        let mut nearby_food = food.iter_mut().find(|(_, _, transform)| {
            transform
                .translation
                .xy()
                .distance(ant_transform.translation.xy())
                < ANT_SENSE_RADIUS
        });

        let nest_distance = ant_transform
            .translation
            .xy()
            .distance(nest_transform.translation.xy());

        if !held_food.empty() && nest_distance < 10.0 {
            let amount = held_food.amount();
            nest.food += amount;
            held_food.remove(amount);
            if nest.food >= 1.0 {
                spawn_ant(
                    &mut commands,
                    &meshes,
                    &colors,
                    ant_transform.translation.x,
                    ant_transform.translation.y,
                    rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI),
                );
                nest.food -= 1.0;
            }
        } else if let Some((entity, mut food, food_transform)) = nearby_food {
            if held_food.empty()
                && food_transform
                    .translation
                    .xy()
                    .distance(ant_transform.translation.xy())
                    < ANT_SEGMENT_RADIUS
            {
                let took = held_food.add(food.amount);
                food.amount -= took;
                if food.amount <= 0.0 {
                    commands.entity(entity).despawn();
                    spawn_food(
                        &mut commands,
                        &meshes,
                        &colors,
                        rng.gen_range(-WORLD_WIDTH / 2.0..WORLD_WIDTH / 2.0),
                        rng.gen_range(-WORLD_HEIGHT / 2.0..WORLD_HEIGHT / 2.0),
                        rng.gen_range(10.0..100.0),
                    );
                    nearby_food = None;
                } else {
                    nearby_food = Some((entity, food, food_transform));
                }
            } else {
                nearby_food = Some((entity, food, food_transform));
            }
        }

        let mut direction = if !held_food.empty() && nest_distance < ANT_SENSE_RADIUS + 10.0 {
            nest_transform.translation.xy() - ant_transform.translation.xy()
        } else if held_food.empty() && nest_distance < ANT_SENSE_RADIUS + 10.0 {
            ant_transform.translation.xy() - nest_transform.translation.xy()
        } else if let Some((_, _, food_transform)) = nearby_food {
            if !held_food.empty() {
                ant_transform.translation.xy() - food_transform.translation.xy()
            } else {
                food_transform.translation.xy() - ant_transform.translation.xy()
            }
        } else {
            nearby_tracks
                .iter()
                .map(|(_, id)| {
                    let (_, track, track_transform) = tracks.get(**id).unwrap();

                    let multiplier = match (&track.kind, !held_food.empty()) {
                        (TrackKind::Nest, false) => -0.1,
                        (TrackKind::Nest, true) => 1.0,
                        (TrackKind::Food, false) => 1.0,
                        (TrackKind::Food, true) => -0.1,
                    };

                    let direction =
                        track_transform.translation.xy() - ant_transform.translation.xy();
                    let direction = if direction.length() > 0.0001 {
                        direction.normalize()
                    } else {
                        Vec2::ZERO
                    };
                    direction * track.concentration * multiplier
                })
                .fold(Vec2::ZERO, |acc, direction| acc + direction)
        };

        if ant_transform.translation.x < -WORLD_WIDTH / 2.0 + ANT_SENSE_RADIUS {
            direction += Vec2::new(10.0, 0.0);
        } else if ant_transform.translation.x > WORLD_WIDTH / 2.0 - ANT_SENSE_RADIUS {
            direction += Vec2::new(-10.0, 0.0);
        }

        if ant_transform.translation.y < -WORLD_HEIGHT / 2.0 + ANT_SENSE_RADIUS {
            direction += Vec2::new(0.0, 10.0);
        } else if ant_transform.translation.y > WORLD_HEIGHT / 2.0 - ANT_SENSE_RADIUS {
            direction += Vec2::new(0.0, -10.0);
        }

        let angle = if direction != Vec2::ZERO {
            direction = direction.normalize();
            let angle = forward.xy().angle_between(direction);
            angle + rng.gen_range(-std::f32::consts::PI / 8.0..std::f32::consts::PI / 8.0)
        } else {
            rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI)
        };

        let max_turn = ANT_ROTATION_SPEED * time.delta_seconds();
        let angle = angle.clamp(-max_turn, max_turn);
        let mult = rng.gen_range(0.5..1.0);
        ant_transform.rotation *= Quat::from_rotation_z(angle * mult);

        let nearest_track = nearby_tracks.iter().find(|(_, id)| {
            let (_, track, track_transform) = tracks.get(**id).unwrap();
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
            track_transform
                .translation
                .xy()
                .distance(ant_transform.translation.xy())
                < TRACK_RADIUS * 2.0
        });

        if let Some((_, id)) = nearest_track {
            let (_, mut track, _) = tracks.get_mut(**id).unwrap();
            track.concentration += ANT_TRACK_CONCENTRATION * time.delta_seconds();
            track.concentration = track.concentration.min(1.0);
        } else {
            spawn_track(
                &mut commands,
                &meshes,
                &colors,
                Transform::from_xyz(
                    ant_transform.translation.x,
                    ant_transform.translation.y,
                    LAYER_TRACK,
                ),
                ANT_TRACK_CONCENTRATION * time.delta_seconds(),
                if !held_food.empty() {
                    TrackKind::Food
                } else {
                    TrackKind::Nest
                },
            );
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
