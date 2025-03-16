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

#[derive(Clone)]
pub struct Forest {
    all_tracks: Vec<Track>,
}

impl Forest {
    pub fn new() -> Self {
        Self {
            all_tracks: Vec::new(),
        }
    }

    pub fn add(&mut self, metrics: [f32; tree::DIMENSIONS], id: u64) {
        let track = Track {
            id: id,
            metrics: metrics,
        };
        self.all_tracks.push(track);
    }

    pub fn sort_by_closest(&self, seeds: &Vec<Track>) -> Vec<Track>{
       let opts = extended_isolation_forest::ForestOptions {
            n_trees: 100,
            sample_size: seeds.len().min(256),
            max_tree_depth: None,
            extension_level: 1,
       };
       let seed_array = &*seeds.iter().map(|s| s.metrics).collect::<Vec<_>>();
       let forest = extended_isolation_forest::Forest::from_slice(seed_array, &opts).unwrap();
       let mut all = self.all_tracks.clone();
       all.sort_by_cached_key(|track| n32(forest.score(&track.metrics) as f32));
       all
    }
}
 