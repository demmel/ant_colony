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
pub struct Ant {
    food: f32,
    satiation: f32,
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
            Ant {
                food: 0.0,
                satiation: 1.0,
            },
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

pub fn update_ants(
    mut commands: Commands,
    time: Res<Time>,
    meshes: Res<Meshes>,
    colors: Res<Colors>,
    mut ants: Query<(Entity, &mut Ant, &mut Transform)>,
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

    for (ant_entity, mut ant, mut ant_transform) in ants.iter_mut() {
        ant.satiation -= 0.01 * time.delta_seconds();
        if ant.food > 0.0 {
            let eats = ((1.0 - ant.satiation) * 0.1 * time.delta_seconds()).min(ant.food);
            ant.satiation += eats;
            ant.food -= eats;
            if ant.food < 0.0001 {
                ant.food = 0.0;
            }
        }
        if ant.satiation < 0.0 {
            commands.entity(ant_entity).despawn_recursive();
            continue;
        }

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

        if ant.food > 0.0 && nest_distance < 10.0 {
            nest.food += ant.food;
            ant.food = 0.0;
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
            if ant.food <= 0.0
                && food_transform
                    .translation
                    .xy()
                    .distance(ant_transform.translation.xy())
                    < ANT_SEGMENT_RADIUS
            {
                food.amount -= 1.0;
                ant.food += 1.0;
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

        let mut direction = if ant.food > 0.0 && nest_distance < ANT_SENSE_RADIUS + 10.0 {
            nest_transform.translation.xy() - ant_transform.translation.xy()
        } else if ant.food <= 0.0 && nest_distance < ANT_SENSE_RADIUS + 10.0 {
            ant_transform.translation.xy() - nest_transform.translation.xy()
        } else if let Some((_, _, food_transform)) = nearby_food {
            if ant.food > 0.0 {
                ant_transform.translation.xy() - food_transform.translation.xy()
            } else {
                food_transform.translation.xy() - ant_transform.translation.xy()
            }
        } else {
            nearby_tracks
                .iter()
                .map(|(_, id)| {
                    let (_, track, track_transform) = tracks.get(**id).unwrap();

                    let multiplier = match (&track.kind, ant.food > 0.0) {
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
            match (&track.kind, ant.food > 0.0) {
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
                if ant.food > 0.0 {
                    TrackKind::Food
                } else {
                    TrackKind::Nest
                },
            );
        }
    }
}

pub fn update_ant_holding_food(
    ants: Query<(&Ant, &Children), Changed<Ant>>,
    mut carried_food: Query<(Entity, &CarriedFood, &mut Visibility)>,
) {
    for (ant, children) in ants.iter() {
        for child in children.iter() {
            if let Ok((_, _, mut visibility)) = carried_food.get_mut(*child) {
                *visibility = if ant.food > 0.0 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}
