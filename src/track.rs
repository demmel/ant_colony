use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    assets::{Colors, Meshes},
    config::{self, FIXED_DELTA_TIME, LAYER_TRACK},
};

#[derive(Debug, PartialEq, Eq)]
pub enum TrackKind {
    Nest,
    Food,
}

#[derive(Component)]
pub struct Track {
    pub concentration: f32,
    pub kind: TrackKind,
}

pub fn spawn_track(
    commands: &mut Commands,
    meshes: &Res<Meshes>,
    colors: &Res<Colors>,
    position: Vec2,
    concentration: f32,
    kind: TrackKind,
) {
    let color_index = (concentration * 100.0).round() as usize;
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.track.clone(),
            material: match kind {
                TrackKind::Nest => colors.nest_tracks[color_index].clone(),
                TrackKind::Food => colors.food_tracks[color_index].clone(),
            },
            transform: Transform::from_xyz(
                position.x,
                position.y,
                LAYER_TRACK
                    + match kind {
                        TrackKind::Nest => 0.0,
                        TrackKind::Food => 0.1,
                    },
            ),
            ..Default::default()
        },
        Track {
            concentration,
            kind,
        },
    ));
}

pub fn decay_tracks(
    mut commands: Commands,
    colors: Res<Colors>,
    mut tracks: Query<(Entity, &mut Track, &mut Handle<ColorMaterial>)>,
) {
    for (entity, mut track, mut material_handle) in tracks.iter_mut() {
        track.concentration *= config::TRACK_CONCENTRAION_FACTOR.powf(FIXED_DELTA_TIME);
        track.concentration = track.concentration.max(0.0);
        if track.concentration < 0.001 {
            commands.entity(entity).despawn();
            continue;
        }
        let material_index = (track.concentration * 100.0).round() as usize;
        *material_handle = match track.kind {
            TrackKind::Nest => colors.nest_tracks[material_index].clone(),
            TrackKind::Food => colors.food_tracks[material_index].clone(),
        };
    }
}
