/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022-2023 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use kiddo::{KdTree, SquaredEuclidean};

pub const DIMENSIONS: usize = 20;

#[derive(Clone)]
pub struct Tree {
    pub tree: KdTree<f32, DIMENSIONS>,
}

pub struct Sim {
    pub id: u64,
    pub sim: f32,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            tree: KdTree::new(),
        }
    }

    pub fn get_similars(&self, seed: &[f32; DIMENSIONS], count: usize) -> Vec<Sim> {
        let mut resp = Vec::<Sim>::new();

        let neighbours =  self.tree.nearest_n::<SquaredEuclidean>(seed, count);// {
        for neighbour in &neighbours {
            let item = Sim {
                id:  neighbour.item,
                sim: neighbour.distance,
            };
            resp.push(item);
        }

        resp
    }
}
