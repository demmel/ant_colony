mod ant;
mod assets;
mod config;
mod food;
mod nest;
mod track;

use ant::{
    decay_satiation, deposit_food, eat_held_food, eat_nest_food, emit_ant_pheromones, pick_up_food,
    rotate_ants, setup_ant_rendering, spawn_ant, starve, update_ant_holding_food, walk_ants,
};
use assets::{Colors, Meshes};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::camera::ScalingMode,
    sprite::MaterialMesh2dBundle,
    window::WindowMode::BorderlessFullscreen,
};
use config::{SimulationConfig, LAYER_DIRT, TICKS_PER_SECOND, TICK_RATE_MULTIPLIER};
use food::{setup_food_rendering, spawn_random_food, update_food_size};
use nest::{emit_nest_pheromones, setup_nest_rendering, spawn_ants_from_nest, spawn_nest};
use rand::prelude::*;
use track::{
    decay_tracks, diffuse_tracks, setup_tracks, setup_tracks_renderin, update_tracks_image,
};

fn main() {
    run_simulation(SimulationConfig::default());
}

fn run_simulation(simulation_config: SimulationConfig) {
    let headless = true;
    let mut app = create_base_app(simulation_config);
    if headless {
        app = augment_headless(app);
    } else {
        app = augment_rendering(app);
    }
    app.run();
}

fn create_base_app(simulation_config: SimulationConfig) -> App {
    let mut app = App::new();

    app.insert_resource(simulation_config)
        .add_systems(Startup, (setup, setup_tracks))
        .add_systems(
            FixedUpdate,
            ((
                (
                    (decay_tracks, diffuse_tracks, emit_nest_pheromones).chain(),
                    ((decay_satiation, eat_held_food), starve).chain(),
                ),
                (
                    walk_ants,
                    (
                        deposit_food,
                        pick_up_food,
                        emit_ant_pheromones,
                        eat_nest_food,
                    ),
                    (rotate_ants, spawn_ants_from_nest),
                )
                    .chain(),
            )
                .chain(),),
        );
    app
}

fn augment_headless(mut app: App) -> App {
    app.add_plugins(MinimalPlugins)
        .insert_resource(Time::<Fixed>::from_hz(
            TICKS_PER_SECOND * TICK_RATE_MULTIPLIER,
        ));
    app
}

fn augment_rendering(mut app: App) -> App {
    app.add_plugins((
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
    .add_systems(Startup, setup_rendering)
    .add_systems(
        Update,
        (
            setup_tracks_renderin,
            (setup_ant_rendering, update_ant_holding_food).chain(),
            (setup_food_rendering, update_food_size).chain(),
            setup_nest_rendering,
            update_tracks_image,
            exit,
        ),
    );
    app
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands, simulation_config: Res<SimulationConfig>) {
    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let x = rng.gen_range(-10.0..10.0);
        let y = rng.gen_range(-10.0..10.0);
        let rotation = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        spawn_ant(
            &mut commands,
            &simulation_config,
            x,
            y,
            rotation,
            simulation_config.ant_kind_gen_config.gen_kind(&mut rng),
        );
    }

    for _ in 0..25 {
        spawn_random_food(&mut commands);
    }

    spawn_nest(&mut commands, 0.0, 0.0);
}

fn setup_rendering(mut commands: Commands, meshes: Res<Meshes>, colors: Res<Colors>) {
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
}

fn exit(keys: Res<ButtonInput<KeyCode>>) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    std::process::exit(0);
}
