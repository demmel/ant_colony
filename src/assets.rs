use bevy::{prelude::*, sprite::Mesh2dHandle};

use crate::config::{
    ANT_ANTENNA_RADIUS, ANT_COLOR, ANT_SEGMENT_RADIUS, DIRT_COLOR, FOOD_COLOR, NEST_COLOR,
    WORLD_HEIGHT, WORLD_WIDTH,
};

#[derive(Resource)]
pub struct Meshes {
    pub food: Mesh2dHandle,
    pub nest: Mesh2dHandle,
    pub ant_antenna: Mesh2dHandle,
    pub ant_segment: Mesh2dHandle,
    pub dirt: Mesh2dHandle,
}

impl FromWorld for Meshes {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self {
            food: Mesh2dHandle(meshes.add(Circle { radius: 1.0 })),
            nest: Mesh2dHandle(meshes.add(Circle { radius: 10.0 })),
            ant_antenna: Mesh2dHandle(meshes.add(Circle {
                radius: ANT_ANTENNA_RADIUS,
            })),
            ant_segment: Mesh2dHandle(meshes.add(Circle {
                radius: ANT_SEGMENT_RADIUS,
            })),
            dirt: Mesh2dHandle(meshes.add(Rectangle::new(WORLD_WIDTH, WORLD_HEIGHT))),
        }
    }
}

#[derive(Resource)]
pub struct Colors {
    pub ant_worker: Handle<ColorMaterial>,
    pub ant_scout: Handle<ColorMaterial>,
    pub dirt: Handle<ColorMaterial>,
    pub food: Handle<ColorMaterial>,
    pub nest: Handle<ColorMaterial>,
}

impl FromWorld for Colors {
    fn from_world(world: &mut World) -> Self {
        let mut colors = world.resource_mut::<Assets<ColorMaterial>>();
        Self {
            ant_worker: colors.add(ANT_COLOR),
            ant_scout: colors.add(Color::srgb(0.0, 0.3, 0.0)),
            dirt: colors.add(DIRT_COLOR),
            nest: colors.add(NEST_COLOR),
            food: colors.add(FOOD_COLOR),
        }
    }
}
