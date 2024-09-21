use std::time::Duration;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::prelude::*;

use crate::{
    ant::spawn_ant,
    assets::{Colors, Meshes},
    config::{SimulationConfig, FIXED_DELTA_TIME, LAYER_NEST, NEST_RADIUS},
    track::Tracks,
};

#[derive(Component)]
pub struct Nest {
    pub food: f32,
}

#[derive(Component)]
pub struct AntSpawner {
    pub timer: Timer,
}

pub fn spawn_nest(
    commands: &mut Commands,
    meshes: &Res<Meshes>,
    colors: &Res<Colors>,
    x: f32,
    y: f32,
) {
    commands.spawn((
        Nest { food: 5.0 },
        MaterialMesh2dBundle {
            mesh: meshes.nest.clone(),
            material: colors.nest.clone(),
            transform: Transform::from_translation(Vec3::new(x, y, LAYER_NEST)),
            ..Default::default()
        },
        AntSpawner {
            timer: Timer::from_seconds(60.0, TimerMode::Repeating),
        },
    ));
}

pub fn spawn_ants_from_nest(
    mut commands: Commands,
    simulation_config: Res<SimulationConfig>,
    meshes: Res<Meshes>,
    colors: Res<Colors>,
    mut query: Query<(&mut Nest, &Transform, &mut AntSpawner)>,
) {
    let mut rng = rand::thread_rng();
    for (mut nest, transform, mut spawner) in query.iter_mut() {
        spawner
            .timer
            .tick(Duration::from_secs_f32(FIXED_DELTA_TIME));
        if !spawner.timer.finished() {
            continue;
        }
        if nest.food < 1.0 {
            continue;
        }
        nest.food -= 1.0;
        let x = transform.translation.x + rng.gen_range(-NEST_RADIUS..NEST_RADIUS);
        let y = transform.translation.y + rng.gen_range(-NEST_RADIUS..NEST_RADIUS);
        let rotation = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        spawn_ant(
            &mut commands,
            &simulation_config,
            &meshes,
            &colors,
            x,
            y,
            rotation,
            simulation_config.ant_kind_gen_config.gen_kind(&mut rng),
        );
    }
}

pub fn emit_nest_pheromones(
    simulation_config: Res<SimulationConfig>,
    mut query: Query<&Transform, With<Nest>>,
    mut tracks: Query<&mut Tracks>,
) {
    let mut tracks = tracks.single_mut();
    for transform in query.iter_mut() {
        let nest_concentration = simulation_config.nest_track_concentration;
        tracks.within_circle_mut(transform.translation.xy(), NEST_RADIUS, |track| {
            track.nest += nest_concentration * FIXED_DELTA_TIME;
        });
    }
}
