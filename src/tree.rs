use kiddo::distance::squared_euclidean;
/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/
use kiddo::KdTree;

pub const DIMENSIONS: usize = 20;

#[derive(Clone)]
pub struct Tree {
    pub tree: KdTree<f32, usize, DIMENSIONS>,
}

pub struct Sim {
    pub id: usize,
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

        match self.tree.nearest(seed, count, &squared_euclidean) {
            Ok(neighbours) => {
                for neighbour in &neighbours {
                    let item = Sim {
                        id: *neighbour.1,
                        sim: neighbour.0,
                    };
                    resp.push(item);
                }
            }
            Err(_e) => {}
        }

        resp
    }
}
