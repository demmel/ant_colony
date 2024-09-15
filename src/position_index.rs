use bevy::prelude::*;
use kdtree::{distance::squared_euclidean, KdTree};

use crate::track::Track;

#[derive(Resource)]
pub struct TrackPositionIndex(KdTree<f32, Entity, [f32; 2]>);

impl Default for TrackPositionIndex {
    fn default() -> Self {
        Self(KdTree::new(2))
    }
}

impl TrackPositionIndex {
    pub fn insert(&mut self, entity: Entity, position: Vec2) {
        self.0.add([position.x, position.y], entity).unwrap();
    }

    pub fn within(
        &self,
        position: Vec2,
        radius: f32,
    ) -> impl Iterator<Item = (f32, &Entity)> + Clone {
        self.0
            .within(
                &[position.x, position.y],
                radius.powi(2),
                &squared_euclidean,
            )
            .unwrap()
            .into_iter()
            .map(|(distance, entity)| (distance.sqrt(), entity))
    }
}

pub fn compute_track_position_index(
    mut track_position_index: ResMut<TrackPositionIndex>,
    tracks: Query<(Entity, &Transform), With<Track>>,
) {
    *track_position_index = TrackPositionIndex::default();
    for (entity, transform) in tracks.iter() {
        track_position_index.insert(entity, transform.translation.xy());
    }
}
