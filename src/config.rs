use bevy::prelude::*;
use enum_ordinalize::Ordinalize;
use rand::prelude::*;

use crate::ant::AntKind;

pub const CLEAR_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
pub const DIRT_COLOR: Color = Color::srgb(155.0 / 255.0, 118.0 / 255.0, 83.0 / 255.0);
pub const ANT_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
pub const NEST_COLOR: Color = Color::srgb(120.0 / 255.0, 82.0 / 255.0, 30.0 / 255.0);
pub const FOOD_COLOR: Color = Color::srgb(126.0 / 255.0, 196.0 / 255.0, 51.0 / 255.0);

pub const WORLD_WIDTH: f32 = 1920.0;
pub const WORLD_HEIGHT: f32 = 1080.0;

pub const ANT_ANTENNA_RADIUS: f32 = 1.0;
pub const ANT_SEGMENT_RADIUS: f32 = 2.0;
pub const ANT_SPEED: f32 = 10.0;
pub const ANT_ROTATION_SPEED: f32 = 2.0 * std::f32::consts::PI;
pub const ANT_ENERGY_LOSS_RATE: f32 = 0.006;
pub const ANT_MAX_ENERGY: f32 = 1.0;

pub const LAYER_DIRT: f32 = 0.0;
pub const LAYER_TRACK: f32 = 1.0;
pub const LAYER_NEST: f32 = 2.0;
pub const LAYER_FOOD: f32 = 3.0;
pub const LAYER_ANT: f32 = 4.0;

pub const TRACK_RADIUS: f32 = 2.0;
pub const TRACK_RESOLUTION: f32 = 4.0;

pub const NEST_RADIUS: f32 = 10.0;

pub const TICKS_PER_SECOND: f64 = 60.0;
pub const FIXED_DELTA_TIME: f32 = 1.0 / TICKS_PER_SECOND as f32;
pub const TICK_RATE_MULTIPLIER: f64 = 4.0;

#[derive(Resource)]
pub struct SimulationConfig {
    pub ant_track_concentration: f32,
    pub ant_sense_distance: f32,
    pub ant_sense_radius: f32,
    pub ant_max_carry: f32,
    pub nest_track_concentration: f32,
    pub track_concentration_factor: f32,
    pub track_diffusion_factor: f32,
    pub ant_kind_gen_config: AntKindGenConfig,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            ant_track_concentration: 0.1,
            ant_sense_distance: 12.0,
            ant_sense_radius: 8.0,
            ant_max_carry: 5.0,
            nest_track_concentration: 0.1,
            track_concentration_factor: 0.99,
            track_diffusion_factor: 0.001,
            ant_kind_gen_config: AntKindGenConfig::new([
                (AntKind::Worker, 1.0),
                (AntKind::Scout, 1.0),
            ]),
        }
    }
}

pub struct AntKindGenConfig {
    weights: [(AntKind, f32); AntKind::VARIANT_COUNT],
}

impl AntKindGenConfig {
    pub fn new(weights: [(AntKind, f32); AntKind::VARIANT_COUNT]) -> Self {
        let mut seen = [false; AntKind::VARIANT_COUNT];
        for (kind, _) in weights.iter() {
            assert!(
                !seen[kind.ordinal() as usize],
                "duplicate ant kind in gen config"
            );
            seen[kind.ordinal() as usize] = true;
        }
        assert!(
            seen.iter().all(|&seen| seen),
            "missing ant kind in gen config"
        );
        Self { weights }
    }

    pub fn gen_kind(&self, rng: &mut impl Rng) -> AntKind {
        self.weights
            .choose_weighted(rng, |(_, weight)| *weight)
            .unwrap()
            .0
    }
}
