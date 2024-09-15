mod ant;
mod assets;
mod config;
mod food;
mod nest;
mod track;

use ant::{
    decay_satiation, eat_held_food, spawn_ant, starve, update_ant_holding_food, update_ants,
};
use assets::{Colors, Meshes};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::camera::ScalingMode,
    sprite::MaterialMesh2dBundle,
    window::WindowMode::BorderlessFullscreen,
};
use config::LAYER_DIRT;
use food::{spawn_food, update_food_size};
use nest::spawn_nest;
use rand::prelude::*;
use track::decay_tracks;

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
        .init_resource::<Meshes>()
        .init_resource::<Colors>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (
                    (
                        decay_tracks,
                        ((decay_satiation, eat_held_food), starve).chain(),
                    ),
                    (update_ants, (update_ant_holding_food, update_food_size)).chain(),
                )
                    .chain(),
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
        spawn_ant(&mut commands, &meshes, &colors, x, y, rotation);
    }

    for _ in 0..10 {
        let x = rng.gen_range(-config::WORLD_WIDTH / 2.0..config::WORLD_WIDTH / 2.0);
        let y = rng.gen_range(-config::WORLD_HEIGHT / 2.0..config::WORLD_HEIGHT / 2.0);
        let amount = rng.gen_range(10.0..100.0);
        spawn_food(&mut commands, &meshes, &colors, x, y, amount);
    }

    spawn_nest(&mut commands, &meshes, &colors, 0.0, 0.0);
}

fn exit(keys: Res<ButtonInput<KeyCode>>) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    std::process::exit(0);
}
