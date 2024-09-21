use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use crate::config::{
    SimulationConfig, FIXED_DELTA_TIME, LAYER_TRACK, TRACK_RESOLUTION, WORLD_HEIGHT, WORLD_WIDTH,
};

pub struct Track {
    pub food: f32,
    pub nest: f32,
}

#[derive(Component)]
pub struct Tracks(Vec<Track>);

impl Tracks {
    pub fn within_circle(&self, center: Vec2, radius: f32) -> impl Iterator<Item = &Track> {
        world_pixels_within_circle(center, radius)
            .map(move |(x, y)| &self.0[x + y * self.width() as usize])
    }

    pub fn within_circle_mut(&mut self, center: Vec2, radius: f32, mut f: impl FnMut(&mut Track)) {
        let width = self.width();
        for (x, y) in world_pixels_within_circle(center, radius) {
            f(&mut self.0[x + y * width as usize]);
        }
    }

    pub fn width(&self) -> usize {
        (WORLD_WIDTH / TRACK_RESOLUTION) as usize
    }

    pub fn height(&self) -> usize {
        (WORLD_HEIGHT / TRACK_RESOLUTION) as usize
    }
}

fn world_pixels_within_circle(center: Vec2, radius: f32) -> impl Iterator<Item = (usize, usize)> {
    let width = WORLD_WIDTH / TRACK_RESOLUTION;
    let height = WORLD_HEIGHT / TRACK_RESOLUTION;
    let tracks_radius = radius / TRACK_RESOLUTION;

    let cx = (center.x + WORLD_WIDTH / 2.0) / TRACK_RESOLUTION;
    let cy = (-center.y + WORLD_HEIGHT / 2.0) / TRACK_RESOLUTION;

    let minx = (cx - tracks_radius).max(0.0).floor() as usize;
    let miny = (cy - tracks_radius).max(0.0).floor() as usize;
    let maxx = (cx + tracks_radius).min(width - 1.0).floor() as usize;
    let maxy = (cy + tracks_radius).min(height - 1.0).floor() as usize;

    (minx..=maxx)
        .flat_map(move |x| (miny..=maxy).map(move |y| (x, y)))
        .filter(move |(x, y)| {
            let dx = cx - *x as f32;
            let dy = cy - *y as f32;
            dx * dx + dy * dy <= tracks_radius * tracks_radius
        })
}

impl Default for Tracks {
    fn default() -> Self {
        Self(
            (0..((WORLD_WIDTH / TRACK_RESOLUTION) as usize
                * (WORLD_HEIGHT / TRACK_RESOLUTION) as usize))
                .map(|_| Track {
                    food: 0.0,
                    nest: 0.0,
                })
                .collect(),
        )
    }
}

pub fn setup_tracks(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let image = Image::new_fill(
        Extent3d {
            width: (WORLD_WIDTH / TRACK_RESOLUTION) as u32,
            height: (WORLD_HEIGHT / TRACK_RESOLUTION) as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 128],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    );
    let texture = textures.add(image);
    commands.spawn((
        Tracks::default(),
        SpriteBundle {
            texture,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, LAYER_TRACK))
                .with_scale(Vec3::new(TRACK_RESOLUTION, TRACK_RESOLUTION, 1.0)),
            ..default()
        },
    ));
}

pub fn decay_tracks(simulation_config: Res<SimulationConfig>, mut tracks: Query<&mut Tracks>) {
    let mut tracks = tracks.single_mut();

    for track in tracks.0.iter_mut() {
        track.food *= simulation_config
            .track_concentration_factor
            .powf(FIXED_DELTA_TIME);
        track.nest *= simulation_config
            .track_concentration_factor
            .powf(FIXED_DELTA_TIME);
    }
}

pub fn diffuse_tracks(simulation_config: Res<SimulationConfig>, mut tracks: Query<&mut Tracks>) {
    let mut tracks = tracks.single_mut();

    for x in 1..tracks.width() as usize - 1 {
        for y in 1..tracks.height() as usize - 1 {
            let i = x + y * tracks.width() as usize;
            let track = &tracks.0[i];

            let mut food = track.food * (1.0 - 4.0 * simulation_config.track_diffusion_factor);
            let mut nest = track.nest * (1.0 - 4.0 * simulation_config.track_diffusion_factor);

            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let neighbor = &tracks.0[(x as isize + dx) as usize
                    + (y as isize + dy) as usize * tracks.width() as usize];
                food += neighbor.food * simulation_config.track_diffusion_factor;
                nest += neighbor.nest * simulation_config.track_diffusion_factor;
            }
            tracks.0[i] = Track { food, nest };
        }
    }
}

pub fn update_tracks_image(
    tracks: Query<(&Tracks, &Handle<Image>)>,
    mut textures: ResMut<Assets<Image>>,
) {
    let (tracks, image) = tracks.single();

    let image = textures.get_mut(image).unwrap();

    for (i, track) in tracks.0.iter().enumerate() {
        let pixel = &mut image.data[i * 4..(i + 1) * 4];

        if track.nest < 0.001 && track.food < 0.001 {
            pixel.copy_from_slice(&[0, 0, 0, 0]);
            continue;
        } else if track.nest < 0.001 {
            pixel.copy_from_slice(&[0, 255, 0, (track.food * 255.0) as u8]);
            continue;
        } else if track.food < 0.001 {
            pixel.copy_from_slice(&[255, 0, 0, (track.nest * 255.0) as u8]);
            continue;
        }

        let nest_over_food = track.nest / track.food;
        let food_over_nest = track.food / track.nest;

        if nest_over_food > 1.0 {
            pixel.copy_from_slice(&[
                255,
                (food_over_nest * 255.0) as u8,
                0,
                (track.nest * 255.0) as u8,
            ]);
        } else {
            pixel.copy_from_slice(&[
                (nest_over_food * 255.0) as u8,
                255,
                0,
                (track.food * 255.0) as u8,
            ]);
        }
    }
}
