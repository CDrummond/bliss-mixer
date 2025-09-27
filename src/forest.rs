/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022-2024 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use crate::tree;
use extended_isolation_forest;
use noisy_float::prelude::*;

#[derive(Clone)]
pub struct Track {
    pub id: u64,
    pub metrics: [f32; tree::DIMENSIONS]
}

pub fn sort_by_closest(details: &tree::AnalysisDetails, seeds: &Vec<Track>) -> Vec<Track> {
    let opts = extended_isolation_forest::ForestOptions {
        n_trees: 1000,
        sample_size: seeds.len().min(256),
        max_tree_depth: None,
        extension_level: 10,
    };
    let seed_array = &*seeds.iter().map(|s| s.metrics).collect::<Vec<_>>();
    let forest = extended_isolation_forest::Forest::from_slice(seed_array, &opts).unwrap();

    let mut idx = 0;
    let mut tracks: Vec<Track> = Vec::new();
    for v in &details.values {
        let track = Track {
            id: details.ids[idx],
            metrics: *v,
        };
        tracks.push(track);
        idx+=1;
    }
    tracks.sort_by_cached_key(|track| n32(forest.score(&track.metrics) as f32));
    tracks
}
 