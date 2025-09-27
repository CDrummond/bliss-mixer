/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022-2025 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use crate::tree;
use extended_isolation_forest;
use noisy_float::prelude::*;
use rayon::prelude::*; // Add rayon for parallelism

#[derive(Clone)]
pub struct Track {
    pub id: u64,
    pub metrics: [f32; tree::DIMENSIONS]
}

// Helper struct for sorting
struct ScoredTrack {
    score: N32,
    track: Track,
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

    // Prepare tracks
    let tracks: Vec<Track> = details
        .values
        .iter()
        .enumerate()
        .map(|(idx, v)| Track {
            id: details.ids[idx],
            metrics: *v,
        })
        .collect();

    // Calculate scores in parallel
    let mut scored_tracks: Vec<ScoredTrack> = tracks
        .into_par_iter()
        .map(|track| {
            let score = n32(forest.score(&track.metrics) as f32);
            ScoredTrack { score, track }
        })
        .collect();

    // Sort by score
    scored_tracks.par_sort_unstable_by_key(|scored| scored.score);

    // Return tracks sorted by score
    scored_tracks.into_iter().map(|scored| scored.track).collect()
}
