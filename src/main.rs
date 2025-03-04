mod ant;
mod assets;
mod config;
mod food;
mod nest;
mod track;

use std::time::Duration;

use ant::{
    decay_satiation, deposit_food, eat_held_food, eat_nest_food, emit_ant_pheromones, pick_up_food,
    rotate_ants, setup_ant_rendering, spawn_ant, starve, update_ant_holding_food, walk_ants,
    HeldFood, Satiation,
};
use assets::{Colors, Meshes};
use bevy::{
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::schedule::ScheduleLabel,
    log::LogPlugin,
    prelude::*,
    render::camera::ScalingMode,
    sprite::MaterialMesh2dBundle,
    time::common_conditions::on_real_timer,
    window::WindowMode::BorderlessFullscreen,
};
use config::{SimulationConfig, LAYER_DIRT, TICKS_PER_SECOND, TICK_RATE_MULTIPLIER};
use food::{setup_food_rendering, spawn_random_food, update_food_size};
use nest::{emit_nest_pheromones, setup_nest_rendering, spawn_ants_from_nest, spawn_nest, Nest};
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

    app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
        .insert_resource(simulation_config)
        .add_systems(Startup, (setup, setup_tracks));
    app
}

fn add_simulation(mut app: App, label: impl ScheduleLabel) -> App {
    app.add_systems(
        label,
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
            log_stats.run_if(on_real_timer(Duration::from_secs(1))),
        )
            .chain(),),
    );
    app
}

fn augment_headless(mut app: App) -> App {
    app.add_plugins((MinimalPlugins, LogPlugin::default(), DiagnosticsPlugin));
    app = add_simulation(app, Update);
    app
}

fn augment_rendering(mut app: App) -> App {
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Ant Colony".into(),
            mode: BorderlessFullscreen,
            ..default()
        }),
        ..default()
    }))
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
    app = add_simulation(app, FixedUpdate);
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

fn log_stats(ants: Query<(&HeldFood, &Satiation), With<ant::Ant>>, nests: Query<&Nest>) {
    let ant_count = ants.iter().count();
    let total_held_food: f32 = ants.iter().map(|(held_food, _)| held_food.amount()).sum();
    let average_held_food = total_held_food / ant_count as f32;
    let total_satiation: f32 = ants.iter().map(|(_, satiation)| satiation.amount()).sum();
    let average_satiation = total_satiation / ant_count as f32;
    let nest_count = nests.iter().count();
    let total_nest_food: f32 = nests.iter().map(|nest| nest.food).sum();
    let average_nest_food = total_nest_food / nest_count as f32;

    info!("ants: {ant_count}");
    info!("average held food: {average_held_food}");
    info!("average satiation: {average_satiation}");
    info!("nests: {nest_count}");
    info!("average nest food: {average_nest_food}");
}
