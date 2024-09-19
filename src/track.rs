use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use crate::config::{
    FIXED_DELTA_TIME, LAYER_TRACK, TRACK_CONCENTRAION_FACTOR, WORLD_HEIGHT, WORLD_WIDTH,
};

pub struct Track {
    pub food: f32,
    pub nest: f32,
}

#[derive(Component)]
pub struct Tracks(Vec<Track>);

impl Tracks {
    pub fn within_circle(
        &self,
        center: Vec2,
        radius: f32,
    ) -> impl Iterator<Item = (usize, usize, &Track)> {
        world_pixels_within_circle(center, radius)
            .map(move |(x, y)| (x, y, &self.0[x + y * WORLD_WIDTH as usize]))
    }

    pub fn within_circle_mut(
        &mut self,
        center: Vec2,
        radius: f32,
        mut f: impl FnMut(usize, usize, &mut Track),
    ) {
        for (x, y) in world_pixels_within_circle(center, radius) {
            f(x, y, &mut self.0[x + y * WORLD_WIDTH as usize]);
        }
    }
}

fn world_pixels_within_circle(center: Vec2, radius: f32) -> impl Iterator<Item = (usize, usize)> {
    let cx = center.x + WORLD_WIDTH / 2.0;
    let cy = -center.y + WORLD_HEIGHT / 2.0;

    let minx = (cx - radius).max(0.0).floor() as usize;
    let miny = (cy - radius).max(0.0).floor() as usize;
    let maxx = (cx + radius).min(WORLD_WIDTH - 1.0).floor() as usize;
    let maxy = (cy + radius).min(WORLD_HEIGHT - 1.0).floor() as usize;

    (minx..=maxx).flat_map(move |x| (miny..=maxy).map(move |y| (x, y)))
}

impl Default for Tracks {
    fn default() -> Self {
        Self(
            (0..(WORLD_WIDTH as usize * WORLD_HEIGHT as usize))
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
            width: WORLD_WIDTH as u32,
            height: WORLD_HEIGHT as u32,
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
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, LAYER_TRACK)),
            ..default()
        },
    ));
}

pub fn decay_tracks(mut tracks: Query<&mut Tracks>) {
    let mut tracks = tracks.single_mut();

    for track in tracks.0.iter_mut() {
        track.food *= TRACK_CONCENTRAION_FACTOR.powf(FIXED_DELTA_TIME);
        track.nest *= TRACK_CONCENTRAION_FACTOR.powf(FIXED_DELTA_TIME);
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
        pixel[0] = (track.nest * 255.0) as u8;
        pixel[1] = (track.food * 255.0) as u8;
        pixel[2] = 0;
        pixel[3] = pixel[0].max(pixel[1]);
    }
}
