use bevy::prelude::*;

pub const CLEAR_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
pub const DIRT_COLOR: Color = Color::srgb(155.0 / 255.0, 118.0 / 255.0, 83.0 / 255.0);
pub const NEST_TRACK_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
pub const FOOD_TRACK_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
pub const ANT_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
pub const NEST_COLOR: Color = Color::srgb(120.0 / 255.0, 82.0 / 255.0, 30.0 / 255.0);
pub const FOOD_COLOR: Color = Color::srgb(126.0 / 255.0, 196.0 / 255.0, 51.0 / 255.0);

pub const WORLD_WIDTH: f32 = 1920.0;
pub const WORLD_HEIGHT: f32 = 1080.0;

pub const ANT_ANTENNA_RADIUS: f32 = 1.0;
pub const ANT_SEGMENT_RADIUS: f32 = 2.0;
pub const ANT_SPEED: f32 = 50.0;
pub const ANT_ROTATION_SPEED: f32 = 2.0 * std::f32::consts::PI;
pub const ANT_TRACK_CONCENTRATION: f32 = 1.0;
pub const ANT_SENSE_RADIUS: f32 = 5.0;
pub const ANT_SENSE_DISTANCE: f32 = 10.0;

pub const LAYER_DIRT: f32 = 0.0;
pub const LAYER_TRACK: f32 = 1.0;
pub const LAYER_NEST: f32 = 2.0;
pub const LAYER_FOOD: f32 = 3.0;
pub const LAYER_ANT: f32 = 4.0;

pub const TRACK_RADIUS: f32 = 2.0;
pub const TRACK_CONCENTRAION_FACTOR: f32 = 0.99;
