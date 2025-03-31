/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022-2024 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use kiddo::{ImmutableKdTree, SquaredEuclidean};
use std::num::NonZero;

pub const DIMENSIONS: usize = 20;

#[derive(Clone)]
pub struct Tree {
    pub tree: ImmutableKdTree<f32, DIMENSIONS>,
    idmap: Vec<u64> // List of rowids
}

pub struct AnalysisDetails {
    pub values: Vec<[f32; DIMENSIONS]>,
    pub ids: Vec<u64>
}

impl AnalysisDetails {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            ids: Vec::new()
        }
    }
}

pub struct Sim {
    pub id: u64,
    pub sim: f32,
}

impl Tree {
    pub fn new(details: &AnalysisDetails) -> Self {
        Self {
            tree: ImmutableKdTree::new_from_slice(&details.values),
            idmap: details.ids.clone()
        }
    }

    pub fn get_similars(&self, seed: &[f32; DIMENSIONS], count: NonZero<usize>) -> Vec<Sim> {
        let mut resp = Vec::<Sim>::new();

        let neighbours = self.tree.nearest_n::<SquaredEuclidean>(seed, count);
        for neighbour in &neighbours {
            let item = Sim {
                id:  self.idmap[neighbour.item as usize],
                sim: neighbour.distance,
            };
            resp.push(item);
        }
        resp
    }
}
