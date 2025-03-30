/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022-2024 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use crate::tree;
use rusqlite::Connection;
use std::collections::HashSet;

pub static mut WEIGHTS: [f32;tree::DIMENSIONS] = [1.0;tree::DIMENSIONS];

pub struct Metadata {
    pub file: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub duration: Option<u32>,
    pub tempo: Option<f32>,
}

pub struct Db {
    pub conn: Connection,
}

pub fn init_weights(weights_str: &String) {
    let vals = weights_str.split(",");
    let mut pos = 0;
    unsafe {
        for val in vals {
            if pos<tree::DIMENSIONS {
                WEIGHTS[pos] = val.parse::<f32>().unwrap();
            }
            pos+=1;
        }
        log::debug!("Weights: {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}", 
            WEIGHTS[0], WEIGHTS[1], WEIGHTS[2], WEIGHTS[3], WEIGHTS[4],
            WEIGHTS[5], WEIGHTS[6], WEIGHTS[7], WEIGHTS[8], WEIGHTS[9],
            WEIGHTS[10], WEIGHTS[11], WEIGHTS[12], WEIGHTS[13], WEIGHTS[14],
            WEIGHTS[15], WEIGHTS[16], WEIGHTS[17], WEIGHTS[18], WEIGHTS[19]);
    }
}

fn adjust(vals: [f32;tree::DIMENSIONS]) -> [f32;tree::DIMENSIONS] {
    let mut adjusted: [f32;tree::DIMENSIONS] = [0.0;tree::DIMENSIONS];
    unsafe {
        for (i, x) in vals.iter().enumerate() {
            adjusted[i] = x * WEIGHTS[i];
        }
    }
    adjusted
}

impl Db {
    pub fn new(path: &String) -> Self {
        Self {
            conn: Connection::open(path).unwrap(),
        }
    }

    pub fn close(self) {
        if let Err(e) = self.conn.close() {
            log::debug!("Error closing database: {:?}", e);
        }
    }

    pub fn load(&self) -> Vec<[f32; tree::DIMENSIONS]> {
        log::debug!("Load tree");
        let mut data: Vec<[f32; tree::DIMENSIONS]> = Vec::new();
        match self.conn.prepare("SELECT Tempo, Zcr, MeanSpectralCentroid, StdDevSpectralCentroid, MeanSpectralRolloff, StdDevSpectralRolloff, MeanSpectralFlatness, StdDevSpectralFlatness, MeanLoudness, StdDevLoudness, Chroma1, Chroma2, Chroma3, Chroma4, Chroma5, Chroma6, Chroma7, Chroma8, Chroma9, Chroma10 FROM Tracks WHERE Ignore IS NOT 1") {
            Ok(mut stmt) => {
                let track_iter = stmt.query_map([], |row| {
                    Ok((row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                        row.get(11)?,
                        row.get(12)?,
                        row.get(13)?,
                        row.get(14)?,
                        row.get(15)?,
                        row.get(16)?,
                        row.get(17)?,
                        row.get(18)?,
                        row.get(19)?
                    ))
                }).unwrap();
                let mut num_loaded = 0;
                for tr in track_iter {
                    let track = tr.unwrap();
                    let vals:[f32;tree::DIMENSIONS] = [
                                track.0,
                                track.1,
                                track.2,
                                track.3,
                                track.4,
                                track.5,
                                track.6,
                                track.7,
                                track.8,
                                track.9,
                                track.10,
                                track.11,
                                track.12,
                                track.13,
                                track.14,
                                track.15,
                                track.16,
                                track.17,
                                track.18,
                                track.19];
                    num_loaded += 1;
                    data.push(adjust(vals));
                }
                log::debug!("Tree loaded {} track(s)", num_loaded);
            }
            Err(e) => { log::error!("Failed to load tree from DB. {}", e); }
        }
        data
    }

    pub fn load_artist_tree(&self, artist: &str) -> Vec<[f32; tree::DIMENSIONS]> {
        log::debug!("Load artist '{}' tree", artist);
        let mut data: Vec<[f32; tree::DIMENSIONS]> = Vec::new();
        match self.conn.prepare("SELECT Tempo, Zcr, MeanSpectralCentroid, StdDevSpectralCentroid, MeanSpectralRolloff, StdDevSpectralRolloff, MeanSpectralFlatness, StdDevSpectralFlatness, MeanLoudness, StdDevLoudness, Chroma1, Chroma2, Chroma3, Chroma4, Chroma5, Chroma6, Chroma7, Chroma8, Chroma9, Chroma10 FROM Tracks WHERE Artist=:artist;") {
            Ok(mut stmt) => {
                let track_iter = stmt.query_map(&[(":artist", &artist)], |row| {
                    Ok((row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                        row.get(11)?,
                        row.get(12)?,
                        row.get(13)?,
                        row.get(14)?,
                        row.get(15)?,
                        row.get(16)?,
                        row.get(17)?,
                        row.get(18)?,
                        row.get(19)?
                    ))
                }).unwrap();
                let mut num_loaded = 0;
                for tr in track_iter {
                    let track = tr.unwrap();
                    let vals:[f32;tree::DIMENSIONS] = [
                                track.0,
                                track.1,
                                track.2,
                                track.3,
                                track.4,
                                track.5,
                                track.6,
                                track.7,
                                track.8,
                                track.9,
                                track.10,
                                track.11,
                                track.12,
                                track.13,
                                track.14,
                                track.15,
                                track.16,
                                track.17,
                                track.18,
                                track.19];
                    num_loaded += 1;
                    data.push(adjust(vals));
                }
                log::debug!("Tree loaded {} track(s)", num_loaded);
            }
            Err(e) => { log::error!("Failed to load tree from DB. {}", e); }
        }
        data
    }

    pub fn get_rowid(&self, path: &str) -> u64 {
        let mut id: u64 = 0;
        if let Ok(mut stmt) = self.conn.prepare("SELECT rowid FROM Tracks WHERE File=:path;") {
            if let Ok(val) = stmt.query_row(&[(":path", &path)], |row| row.get(0)) {
                id = val;
            }
        }
        id
    }

    pub fn get_all_genres(&self) -> HashSet<String> {
        log::debug!("getting genres from db.");
        let mut all_available_genres = HashSet::new();

        match self.conn.prepare("SELECT DISTINCT Genre FROM Tracks WHERE ignore IS NOT 1;") {
            Ok(mut stmt) => match stmt.query_map([], |row| Ok(row.get::<_, Option<String>>(0)?)) {
                Ok(column) => {
                    for item in column {
                        let item_content = item.unwrap().unwrap();
                        let item_genres: Vec<&str> = item_content.split(";").collect();
                        for genre in item_genres {
                            let trimmed_genre = genre.trim();
                            if !trimmed_genre.is_empty() {
                                all_available_genres.insert(String::from(trimmed_genre));
                            }
                        }
                    }
                }
                Err(e) => { log::debug!("Failed to read all genres: {}", e); }
            }
            Err(e) => { log::debug!("Failed to read all genres: {}", e); }
        }
        all_available_genres
    }

    pub fn get_metadata(&self, id: u64) -> Result<Metadata, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT File, Title, Artist, AlbumArtist, Album, Genre, Duration, Tempo FROM Tracks WHERE rowid=:rowid;")?;
        let row = stmt.query_row(&[(":rowid", &id)], |row| {
                Ok(Metadata {
                    file: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    album_artist: row.get(3)?,
                    album: row.get(4)?,
                    genre: row.get(5)?,
                    duration: row.get(6)?,
                    tempo: row.get(7)?,
                })
            }).unwrap();
        Ok(row)
    }

    pub fn get_metrics(&self, id: u64) -> Result<[f32; tree::DIMENSIONS], rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT Tempo, Zcr, MeanSpectralCentroid, StdDevSpectralCentroid, MeanSpectralRolloff, StdDevSpectralRolloff, MeanSpectralFlatness, StdDevSpectralFlatness, MeanLoudness, StdDevLoudness, Chroma1, Chroma2, Chroma3, Chroma4, Chroma5, Chroma6, Chroma7, Chroma8, Chroma9, Chroma10 FROM Tracks WHERE rowid=:rowid;").unwrap();
        let row = stmt.query_row(&[(":rowid", &id)], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                    row.get(19)?,
                ))
            }).unwrap();
        let metrics: [f32; tree::DIMENSIONS] = [
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8, row.9, row.10, row.11,
            row.12, row.13, row.14, row.15, row.16, row.17, row.18, row.19,
        ];
        Ok(adjust(metrics))
    }
}
