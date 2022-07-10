/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use crate::tree;
use rusqlite::Connection;

pub struct Metadata {
    pub file: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub duration: Option<u32>,
}

pub struct Db {
    pub conn: Connection,
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

    pub fn load_tree(&self, tree: &mut tree::Tree) {
        log::debug!("Load tree");
        match self.conn.prepare("SELECT Tempo, Zcr, MeanSpectralCentroid, StdDevSpectralCentroid, MeanSpectralRolloff, StdDevSpectralRolloff, MeanSpectralFlatness, StdDevSpectralFlatness, MeanLoudness, StdDevLoudness, Chroma1, Chroma2, Chroma3, Chroma4, Chroma5, Chroma6, Chroma7, Chroma8, Chroma9, Chroma10, rowid FROM Tracks WHERE Ignore IS NOT 1") {
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
                        row.get(19)?,
                        row.get(20)?
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
                    if let Err(e) = tree.tree.add(&vals, track.20) {
                        log::debug!("Error adding track to tree: {}", e);
                    }
                }
                log::debug!("Tree loaded {} track(s)", num_loaded);
            }
            Err(e) => { log::error!("Failed to load tree from DB. {}", e); }
        }
    }

    pub fn load_artist_tree(&self, tree: &mut tree::Tree, artist: &str) {
        log::debug!("Load artist '{}' tree", artist);
        match self.conn.prepare("SELECT Tempo, Zcr, MeanSpectralCentroid, StdDevSpectralCentroid, MeanSpectralRolloff, StdDevSpectralRolloff, MeanSpectralFlatness, StdDevSpectralFlatness, MeanLoudness, StdDevLoudness, Chroma1, Chroma2, Chroma3, Chroma4, Chroma5, Chroma6, Chroma7, Chroma8, Chroma9, Chroma10, rowid FROM Tracks WHERE Artist=:artist;") {
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
                        row.get(19)?,
                        row.get(20)?
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
                    if let Err(e) = tree.tree.add(&vals, track.20) {
                        log::debug!("Error adding track to tree: {}", e);
                    }
                }
                log::debug!("Tree loaded {} track(s)", num_loaded);
            }
            Err(e) => { log::error!("Failed to load tree from DB. {}", e); }
        }
    }

    pub fn get_rowid(&self, path: &str) -> usize {
        let mut id: usize = 0;
        if let Ok(mut stmt) = self.conn.prepare("SELECT rowid FROM Tracks WHERE File=:path;") {
            if let Ok(val) = stmt.query_row(&[(":path", &path)], |row| row.get(0)) {
                id = val;
            }
        }
        id
    }

    pub fn get_metadata(&self, id: usize) -> Result<Metadata, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT File, Title, Artist, AlbumArtist, Album, Genre, Duration FROM Tracks WHERE rowid=:rowid;")?;
        let row = stmt.query_row(&[(":rowid", &id)], |row| {
                Ok(Metadata {
                    file: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    album_artist: row.get(3)?,
                    album: row.get(4)?,
                    genre: row.get(5)?,
                    duration: row.get(6)?,
                })
            })
            .unwrap();
        Ok(row)
    }

    pub fn get_metrics(&self, id: usize) -> Result<[f32; tree::DIMENSIONS], rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT Tempo, Zcr, MeanSpectralCentroid, StdDevSpectralCentroid, MeanSpectralRolloff, StdDevSpectralRolloff, MeanSpectralFlatness, StdDevSpectralFlatness, MeanLoudness, StdDevLoudness, Chroma1, Chroma2, Chroma3, Chroma4, Chroma5, Chroma6, Chroma7, Chroma8, Chroma9, Chroma10 FROM Tracks WHERE rowid=:rowid;").unwrap();
        let row = stmt
            .query_row(&[(":rowid", &id)], |row| {
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
            })
            .unwrap();
        let metrics: [f32; tree::DIMENSIONS] = [
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8, row.9, row.10, row.11,
            row.12, row.13, row.14, row.15, row.16, row.17, row.18, row.19,
        ];
        Ok(metrics)
    }
}
