mod ant;
mod assets;
mod config;
mod food;
mod nest;
mod track;

use ant::{
    decay_satiation, deposit_food, eat_held_food, eat_nest_food, emit_pheromones, pick_up_food,
    rotate_ants, spawn_ant, starve, update_ant_holding_food, walk_ants,
};
use assets::{Colors, Meshes};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::camera::ScalingMode,
    sprite::MaterialMesh2dBundle,
    window::WindowMode::BorderlessFullscreen,
};
use config::{LAYER_DIRT, TICKS_PER_SECOND, TICK_RATE_MULTIPLIER};
use food::{spawn_random_food, update_food_size};
use nest::{spawn_ants_from_nest, spawn_nest};
use rand::prelude::*;
use track::{decay_tracks, diffuse_tracks, setup_tracks, update_tracks_image};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ant Colony".into(),
                    mode: BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(ClearColor(config::CLEAR_COLOR))
        .insert_resource(Time::<Fixed>::from_hz(
            TICKS_PER_SECOND * TICK_RATE_MULTIPLIER,
        ))
        .init_resource::<Meshes>()
        .init_resource::<Colors>()
        .add_systems(Startup, (setup, setup_tracks))
        .add_systems(
            FixedUpdate,
            ((
                (
                    (decay_tracks, diffuse_tracks).chain(),
                    ((decay_satiation, eat_held_food), starve).chain(),
                ),
                (
                    walk_ants,
                    (deposit_food, pick_up_food, emit_pheromones, eat_nest_food),
                    (rotate_ants, spawn_ants_from_nest),
                )
                    .chain(),
            )
                .chain(),),
        )
        .add_systems(
            Update,
            (
                update_ant_holding_food,
                update_food_size,
                update_tracks_image,
                exit,
            ),
        )
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands, meshes: Res<Meshes>, colors: Res<Colors>) {
    let mut rng = rand::thread_rng();

    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                far: 1000.0,
                near: -1000.0,
                scale: 1.0,
                scaling_mode: ScalingMode::AutoMax {
                    max_width: config::WORLD_WIDTH,
                    max_height: config::WORLD_HEIGHT,
                },
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.dirt.clone(),
        material: colors.dirt.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, LAYER_DIRT)),
        ..Default::default()
    });

    for _ in 0..100 {
        let x = rng.gen_range(-10.0..10.0);
        let y = rng.gen_range(-10.0..10.0);
        let rotation = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        spawn_ant(
            &mut commands,
            &meshes,
            &colors,
            x,
            y,
            rotation,
            match rng.gen_range(0..100) {
                0..50 => ant::AntKind::Scout,
                50..100 => ant::AntKind::Worker,
                _ => unreachable!(),
            },
        );
    }

    for _ in 0..5 {
        spawn_random_food(&mut commands, &meshes, &colors);
    }

    spawn_nest(&mut commands, &meshes, &colors, 0.0, 0.0);
}

fn exit(keys: Res<ButtonInput<KeyCode>>) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    std::process::exit(0);
}
