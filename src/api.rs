/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022-2024 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use crate::db;
use crate::forest;
use crate::tree;
use actix_web::{web, HttpRequest, Responder};
use chrono::Datelike;
use globset::Glob;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::num::NonZero;

const CHRISTMAS: &str = "christmas";
const VARIOUS: &str = "various";
const VARIOUS_ARTISTS: &str = "various artists";
const MIN_FOR_FOREST: usize = 4;
const MIN_COUNT: usize = 1;
const MAX_COUNT: usize = 50;
const MIN_NUM_SIM: usize = 5000;
const MAX_ARTIST_TRACKS: usize = 5;
// KDTree is returning squared-euc distance. So max diff = sqr(0.1) = 0.01
const MAX_ARTIST_TRACK_SIM_DIFF: f32 = 0.01;

#[derive(Deserialize)]
pub struct MixParams {
    count: Option<u16>,
    filtergenre: Option<u16>,
    filterxmas: Option<u16>,
    min: Option<u32>,
    max: Option<u32>,
    maxbpmdiff: Option<i16>,
    tracks: Vec<String>,
    previous: Option<Vec<String>>,
    shuffle: Option<u16>,
    norepart: Option<u16>,
    norepalb: Option<u16>,
    genregroups: Vec<Vec<String>>,
    forest: Option<u16>,
}

#[derive(Deserialize)]
pub struct ListParams {
    count: Option<u16>,
    filtergenre: Option<u16>,
    filterxmas: Option<u16>,
    min: Option<u32>,
    max: Option<u32>,
    maxbpmdiff: Option<i16>,
    track: String,
    genregroups: Vec<Vec<String>>,
    byartist: i16,
}

#[derive(Clone)]
struct Track {
    found: bool,
    id: u64,
    file: String,
    title: String,
    // Original artist, so that can use for api/list
    orig_artist: String,
    artist: String,
    album_artist: String,
    album: String,
    genres: HashSet<String>,
    duration: u32,
    sim: f32,
    is_various: bool,
    bpm: i16
}

#[derive(Clone)]
struct TrackFile {
    file: String,
    sim: f32,
}

struct MatchedArtist {
    pos: usize,
    tracks: Vec<TrackFile>,
}

fn get_track_from_id(db: &db::Db, id: u64) -> Track {
    let mut info = Track {
        found: false,
        id: 0,
        file: String::new(),
        title: String::new(),
        artist: String::new(),
        orig_artist: String::new(),
        album_artist: String::new(),
        album: String::new(),
        genres: HashSet::new(),
        duration: 0,
        sim: 0.,
        is_various: false,
        bpm: 0,
    };

    match db.get_metadata(id) {
        Ok(m) => {
            info.id = id;
            info.found = true;
            info.file = m.file;
            info.title = m.title.unwrap_or_default().to_lowercase();
            info.orig_artist = m.artist.unwrap_or_default();
            info.artist = info.orig_artist.to_lowercase();
            info.album_artist = m.album_artist.unwrap_or_default().to_lowercase();
            if info.album_artist.is_empty() {
                info.album = m.album.unwrap_or_default().to_lowercase() + "::" + &info.artist;
            } else {
                info.is_various =
                    info.album_artist == VARIOUS || info.album_artist == VARIOUS_ARTISTS;
                info.album = m.album.unwrap_or_default().to_lowercase() + "::" + &info.album_artist;
            }
            let genre = m.genre.unwrap_or_default();
            let genres: Vec<&str> = genre.split(';').collect();
            for g in genres {
                let trimmed = g.trim();
                if !trimmed.is_empty() {
                    info.genres.insert(String::from(trimmed.to_lowercase()));
                }
            }
            info.duration = m.duration.unwrap_or(0);
            info.bpm = (((m.tempo.unwrap_or(0.0)+1.0)*206.0)/2.0) as i16;
        }
        Err(e) => {
            log::error!("Failed to read metadata. {}", e);
        }
    }
    info
}

fn get_track(db: &db::Db, track: &str) -> Track {
    let mut info = Track {
        found: false,
        id: 0,
        file: String::new(),
        title: String::new(),
        artist: String::new(),
        orig_artist: String::new(),
        album_artist: String::new(),
        album: String::new(),
        genres: HashSet::new(),
        duration: 0,
        sim: 0.,
        is_various: false,
        bpm: 0
    };

    let id = db.get_rowid(track);
    if id > 0 {
        info = get_track_from_id(db, id);
        if !info.found {
            log::warn!("Could not find '{}' in DB", track);
        }
    } else {
        log::error!("Track '{}' not found in DB", track);
    }
    info
}

fn get_genres(genregroups: &Vec<HashSet<String>>, track_genres: &HashSet<String>) -> HashSet<String> {
    let mut genres: HashSet<String> = HashSet::new();

    for group in genregroups {
        if track_genres.is_subset(group) {
            for genre in group {
                genres.insert(genre.to_string());
            }
        }
    }
    genres
}

fn filter_genre(track_genres: &HashSet<String>, acceptable_genres: &HashSet<String>, all_genres_from_groups: &HashSet<String>) -> bool {
    let mut rv: bool = false;
    if track_genres.is_empty() {
        rv = false;
    } else if acceptable_genres.is_empty() {
        // Seed is not in a genre group...
        if !all_genres_from_groups.is_empty() && !track_genres.is_disjoint(all_genres_from_groups) {
            // ...but candidate track is in a genre group - so filter out track
            rv = true
        }
    } else {
        rv = track_genres.is_disjoint(acceptable_genres)
    }
    rv
}

fn expand_globbed_genres(genregroups: &Vec<Vec<String>>, all_db_genres: &HashSet<String>) -> Vec<HashSet<String>> {
    let mut expanded: Vec<HashSet<String>> = Vec::new();

    for group in genregroups {
        let mut gset: HashSet<String> = HashSet::new();
        for genre in group {
            let lgenre = genre.to_lowercase();
            let glob = Glob::new(&lgenre).unwrap().compile_matcher();
            for item in all_db_genres {
                if glob.is_match(item) {
                    gset.insert(item.to_string());
                }
            }
        }
        expanded.push(gset);
    }
    expanded
}

fn log(reason: &str, trk: &Track) {
    log::debug!("{} File:{}, Title:{}, Album/Artist:{}, Dur:{}, Sim:{:.18}, Genres:{:?}, BPM:{}", reason, trk.file, trk.title, trk.album, trk.duration, trk.sim, trk.genres, trk.bpm);
}

pub async fn mix(req: HttpRequest, payload: web::Json<MixParams>) -> impl Responder {
    let tree = req.app_data::<web::Data<tree::Tree>>().unwrap();
    let all_db_genres = req.app_data::<web::Data<HashSet<String>>>().unwrap();
    let db_path = req.app_data::<web::Data<String>>().unwrap();
    let db = db::Db::new(db_path);
    let mut count = payload.count.unwrap_or(5) as usize;
    let filtergenre = payload.filtergenre.unwrap_or(0);
    let mut filterxmas = payload.filterxmas.unwrap_or(0);
    let min = payload.min.unwrap_or(0);
    let max = payload.max.unwrap_or(0);
    let maxbpmdiff = payload.maxbpmdiff.unwrap_or(0);
    let shuffle = payload.shuffle.unwrap_or(0);
    let norepart = payload.norepart.unwrap_or(0);
    let norepalb = payload.norepalb.unwrap_or(0);
    let genregroups = expand_globbed_genres(&payload.genregroups, &all_db_genres);
    let mut useforest = payload.forest.unwrap_or(0);
    let mut seeds: Vec<Track> = Vec::new();
    // Tracks filtered out due to title matching seed or chosen track
    let mut filter_out_titles: HashSet<String> = HashSet::new();
    // Tracks filtered out due to artist matching seed or chosen track
    let mut filter_out_artists: HashSet<String> = HashSet::new();
    // Tracks filtered out due to album matching seed or chosen track
    let mut filter_out_albums: HashSet<String> = HashSet::new();
    // IDs of seeds, previous, and chosen tracks - to prevent duplicates
    let mut filter_out_ids: HashSet<u64> = HashSet::new();
    // All acceptable genres
    let mut acceptable_genres: HashSet<String> = HashSet::new();
    // All genres that are in a group, genres not in a group are in 'other genres'
    let mut all_genres_from_groups: HashSet<String> = HashSet::new();
    // Albums from matching tracks. Don't want same album chosen twice, even if
    // norepalb is 0 or album is a VA album.
    let mut chosen_albums: HashSet<String> = HashSet::new();

    if count < MIN_COUNT {
        count = MIN_COUNT;
    } else if count > MAX_COUNT {
        count = MAX_COUNT;
    }

    if filterxmas == 1 && chrono::Local::now().month() == 12 {
        filterxmas = 0;
    }

    for group in &genregroups {
        for genre in group {
            all_genres_from_groups.insert(genre.to_string());
        }
    }

    // Find previous in DB
    if let Some(previous) = &payload.previous {
        let mut pcount = 0;
        for track in previous {
            let trk: Track = get_track(&db, track);
            if !trk.found {
                continue;
            }
            filter_out_ids.insert(trk.id);
            if !trk.title.is_empty() {
                filter_out_titles.insert(trk.title);
            }
            if pcount < norepart && !trk.artist.is_empty() {
                filter_out_artists.insert(trk.artist);
            }
            if pcount < norepalb && !trk.album.is_empty() {
                filter_out_albums.insert(trk.album);
            }
            pcount += 1;
            if filtergenre == 1 && !trk.genres.is_empty() {
                let genres = get_genres(&genregroups, &trk.genres);
                acceptable_genres.extend(genres);
            }
        }
    }

    let mut minbpm: i16 = 500;
    let mut maxbpm: i16 = 0;

    // Find seeds in DB
    for track in &payload.tracks {
        let trk: Track = get_track(&db, track);
        if !trk.found {
            continue;
        }
        if filtergenre == 1 {
            let genres = get_genres(&genregroups, &trk.genres);
            acceptable_genres.extend(genres.clone());
        }
        filter_out_ids.insert(trk.id);
        if !trk.title.is_empty() {
            filter_out_titles.insert(trk.title.clone());
        }
        if trk.bpm>maxbpm {
            maxbpm = trk.bpm
        }
        if trk.bpm<minbpm {
            minbpm = trk.bpm
        }
        seeds.push(trk);
    }

    log::debug!("filtergenre:{}, filterxmas:{}, min:{}, max:{}, shuffle:{}, norepart:{}, norepalb:{}", filtergenre, filterxmas, min, max, shuffle, norepart, norepalb);

    if filtergenre == 1 {
        log::debug!("Acceptable genres: {:?}", acceptable_genres);
    }

    // List of tracks that have passed filtering
    let mut chosen: Vec<TrackFile> = Vec::new();
    // List of tracks that were filtered due to meta-data
    let mut filtered: Vec<TrackFile> = Vec::new();
    // Map of artist name to list of similar tracks. If we are shuffling
    // and an artist has multiple similar tracks we will choose one at random
    let mut matched_artists: HashMap<String, MatchedArtist> = HashMap::new();
    // How many simlar tracks should we locate in total?
    let mut similarity_count: usize = count;
    if shuffle == 1 && count < 20 {
        similarity_count = count * 5;
    }

    let mut fseeds: Vec<forest::Track> = Vec::new();
    if useforest>0 && seeds.len()>MIN_FOR_FOREST {
        for seed in seeds.clone() {
            if let Ok(metrics) = db.get_metrics(seed.id) {
                let track = forest::Track {
                    id: seed.id,
                    metrics: metrics,
                };
                fseeds.push(track)
            }
        }
    }

    if fseeds.len()>MIN_FOR_FOREST {
        log::debug!("Using extended isolation forest algorithm");
        let mut forest:tree::AnalysisDetails = tree::AnalysisDetails::new();
        let mut forest_ids: HashSet<u64> = HashSet::new();
        let num_per_file = ((10000/fseeds.len()) as usize).min(1000);
        for seed in seeds {
            if let Ok(metrics) = db.get_metrics(seed.id) {
                log::debug!("Looking for {} tracks similar to '{}'", num_per_file, seed.file);
                let sim_tracks = tree.get_similars(&metrics, NonZero::new(num_per_file).unwrap());
                for sim_track in sim_tracks {
                    if !forest_ids.contains(&sim_track.id) {
                        if let Ok(smetrics) = db.get_metrics(sim_track.id) {
                            forest.values.push(smetrics);
                            forest.ids.push(sim_track.id);
                            forest_ids.insert(sim_track.id);
                        }
                    }
                }
            }
        }

        log::debug!("Forest size: {}", forest.values.len());
        for track in forest::sort_by_closest(&forest, &fseeds) {
            if filter_out_ids.contains(&track.id) {
                continue;
            }
            filter_out_ids.insert(track.id);
            let trk: Track = get_track_from_id(&db, track.id);
            if (min > 0 && trk.duration < min) || (max > 0 && trk.duration > max) {
                log("DISCARD(duration)", &trk);
                continue;
            }
            if maxbpmdiff > 0 && minbpm > 0 && maxbpm > 0 && trk.bpm>0 && (trk.bpm<(minbpm-maxbpmdiff) || trk.bpm>(maxbpm+maxbpmdiff)) {
                log("DISCARD(bpm)", &trk);
                continue;
            }
            if filtergenre == 1 && filter_genre(&trk.genres, &acceptable_genres, &all_genres_from_groups) {
                log("DISCARD(genre)", &trk);
                continue;
            }
            if filterxmas == 1 && trk.genres.contains(CHRISTMAS) {
                log("DISCARD(christmas)", &trk);
                continue;
            }
            if chosen_albums.contains(&trk.album) {
                log("DISCARD(album)", &trk);
                continue;
            }
            let track_file = TrackFile {
                file: trk.file.clone(),
                sim: 1.0,
            };
            if norepart > 0 && filter_out_artists.contains(&trk.artist) {
                log("FILTER(artist)", &trk);
                filtered.push(track_file);
                continue;
            }
            if !trk.is_various && norepalb > 0 && filter_out_albums.contains(&trk.album) {
                log("FILTER(album)", &trk);
                filtered.push(track_file);
                continue;
            }
            if filter_out_titles.contains(&trk.title) {
                log("FILTER(title)", &trk);
                filtered.push(track_file);
                continue;
            }
            log("USABLE", &trk);
            filter_out_titles.insert(trk.title.clone());
            if norepart > 0 {
                filter_out_artists.insert(trk.artist.clone());
            }
            if norepalb > 0 {
                filter_out_albums.insert(trk.album.clone());
            }
            chosen_albums.insert(trk.album.clone());
            chosen.push(track_file.clone());

            if chosen.len()>=similarity_count {
                break;
            }
        }
    } else {
        log::debug!("Using standard algorithm");
        useforest = 0;

        // Map of id to its position in chosen. This is used incase a track
        // matches multiple seeds. In which case we want the sim value to
        // be the lowest of its matches
        let mut id_to_pos: HashMap<u64, usize> = HashMap::new();

        // How many tracks per seed?
        let mut tracks_per_seed = similarity_count;
        if similarity_count < 15 {
            tracks_per_seed = similarity_count * 3;
        }

        // How many similar tracks should we get from KDTree?
        let mut num_sim = count * seeds.len() * 50;
        if num_sim < MIN_NUM_SIM {
            num_sim = MIN_NUM_SIM;
        }

        for seed in seeds {
            let mut accepted_for_seed = 0;
            if let Ok(metrics) = db.get_metrics(seed.id) {
                log::debug!("Looking for tracks similar to '{}'", seed.file);
                let sim_tracks = tree.get_similars(&metrics, NonZero::new(num_sim).unwrap());
                for sim_track in sim_tracks {
                    if filter_out_ids.contains(&sim_track.id) {
                        // Seen from previous seed, so set similarity to lowest value
                        match id_to_pos.get(&sim_track.id) {
                            Some(pos) => {
                                if chosen[*pos].sim > sim_track.sim {
                                    chosen[*pos].sim = sim_track.sim;
                                }
                            }
                            None => {}
                        }
                    } else {
                        filter_out_ids.insert(sim_track.id);
                        let mut trk: Track = get_track_from_id(&db, sim_track.id);
                        trk.sim = sim_track.sim;
                        if (min > 0 && trk.duration < min) || (max > 0 && trk.duration > max) {
                            log("DISCARD(duration)", &trk);
                            continue;
                        }
                        if maxbpmdiff > 0 && trk.bpm> 0 && seed.bpm>0 && (trk.bpm-seed.bpm).abs()>maxbpmdiff {
                            log("DISCARD(bpm)", &trk);
                            continue;
                        }
                        if filtergenre == 1 && filter_genre(&trk.genres, &acceptable_genres, &all_genres_from_groups) {
                            log("DISCARD(genre)", &trk);
                            continue;
                        }
                        if filterxmas == 1 && trk.genres.contains(CHRISTMAS) {
                            log("DISCARD(christmas)", &trk);
                            continue;
                        }
                        if chosen_albums.contains(&trk.album) {
                            log("DISCARD(album)", &trk);
                            continue;
                        }
                        let track_file = TrackFile {
                            file: trk.file.clone(),
                            sim: trk.sim,
                        };
                        if norepart > 0 && filter_out_artists.contains(&trk.artist) {
                            log("FILTER(artist)", &trk);

                            if shuffle == 1 {
                                // We have seen this artist before. If this track is close in similarity
                                // to the first from this artist then store it - we will choose a random
                                // track later.
                                match matched_artists.get_mut(&trk.artist) {
                                    Some(artist) => {
                                        if artist.tracks.len() < MAX_ARTIST_TRACKS && (sim_track.sim - artist.tracks[0].sim).abs() < MAX_ARTIST_TRACK_SIM_DIFF {
                                            artist.tracks.push(track_file.clone())
                                        }
                                    }
                                    None => {}
                                }
                            }

                            filtered.push(track_file);
                            continue;
                        }
                        if !trk.is_various && norepalb > 0 && filter_out_albums.contains(&trk.album) {
                            log("FILTER(album)", &trk);
                            filtered.push(track_file);
                            continue;
                        }
                        if filter_out_titles.contains(&trk.title) {
                            log("FILTER(title)", &trk);
                            filtered.push(track_file);
                            continue;
                        }
                        log("USABLE", &trk);
                        filter_out_titles.insert(trk.title.clone());
                        if norepart > 0 {
                            filter_out_artists.insert(trk.artist.clone());
                        }
                        if norepalb > 0 {
                            filter_out_albums.insert(trk.album.clone());
                        }
                        chosen_albums.insert(trk.album.clone());
                        id_to_pos.insert(trk.id, chosen.len());
                        chosen.push(track_file.clone());

                        if shuffle == 1 {
                            // Store this track linked to artist. Next time we see artist we
                            // will extend this list of tracks so that we can choose a random
                            // one later.
                            let mut matched_artist = MatchedArtist {
                                pos: chosen.len() - 1,
                                tracks: Vec::new(),
                            };
                            matched_artist.tracks.push(track_file);
                            matched_artists.insert(trk.artist.clone(), matched_artist);
                        }

                        accepted_for_seed += 1;
                        if accepted_for_seed >= tracks_per_seed {
                            break;
                        }
                    }
                }
            }
        }
    }

    db.close();

    log::debug!("similar_tracks: {}, filtered_tracks:{}", chosen.len(), filtered.len());
    if useforest!=1 {
        let mut min_count: usize = 2;
        if min_count > count {
            min_count = count;
        }

        if shuffle == 1  {
            // For each artist that had multiple similar tracks, choose one at random
            for (name, info) in matched_artists {
                if info.tracks.len() > 1 {
                    log::debug!("Choosing random track for {} ({} tracks)", name, info.tracks.len());
                    match info.tracks.choose(&mut thread_rng()) {
                        Some(trk) => {
                            chosen[info.pos].file = trk.file.clone();
                        }
                        None => {}
                    }
                }
            }
        }

        // Too few tracks? Choose some from filtered...
        if chosen.len() < min_count && !filtered.is_empty() {
            filtered.sort_by(|a, b| a.sim.partial_cmp(&b.sim).unwrap());
            while chosen.len() < min_count && !filtered.is_empty() {
                chosen.push(filtered.remove(0));
            }
        }
    }

    // Sort by similarity
    chosen.sort_by(|a, b| a.sim.partial_cmp(&b.sim).unwrap());

    if shuffle == 1 {
        // Take top 'similarity_count' tracks
        chosen.truncate(similarity_count);
        // Shuffle
        chosen.shuffle(&mut thread_rng());
    }

    // Take 'count' tracks
    chosen.truncate(count);

    let mut resp = String::new();
    for track in chosen {
        resp += &track.file;
        resp += "\n";
    }
    resp
}

pub async fn list(req: HttpRequest, payload: web::Json<ListParams>) -> impl Responder {
    let db_path = req.app_data::<web::Data<String>>().unwrap();
    let all_db_genres = req.app_data::<web::Data<HashSet<String>>>().unwrap();
    let db = db::Db::new(db_path);
    let mut count = payload.count.unwrap_or(5) as usize;
    let filtergenre = payload.filtergenre.unwrap_or(0);
    let mut filterxmas = payload.filterxmas.unwrap_or(0);
    let min = payload.min.unwrap_or(0);
    let max = payload.max.unwrap_or(0);
    let maxbpmdiff = payload.maxbpmdiff.unwrap_or(0);
    let track = &payload.track;
    let byartist = payload.byartist;
    let genregroups = expand_globbed_genres(&payload.genregroups, &all_db_genres);
    let mut acceptable_genres: HashSet<String> = HashSet::new();
    let mut all_genres_from_groups: HashSet<String> = HashSet::new();
    let mut chosen: Vec<String> = Vec::new();
    let mut filter_out_titles: HashSet<String> = HashSet::new();

    if filterxmas == 1 && chrono::Local::now().month() == 12 {
         filterxmas = 0;
     }

    if count < MIN_COUNT {
        count = MIN_COUNT;
    } else if count > MAX_COUNT {
        count = MAX_COUNT;
    }

    log::debug!("Looking for tracks similar to '{}'", track);
    let seed: Track = get_track(&db, track);
    if seed.found {
        if filtergenre == 1 {
            for group in &genregroups {
                for genre in group {
                    all_genres_from_groups.insert(genre.to_string());
                }
            }
            if !seed.genres.is_empty() {
                let genres = get_genres(&genregroups, &seed.genres);
                acceptable_genres.extend(genres);
            }
        }
        filter_out_titles.insert(seed.title);
        if let Ok(metrics) = db.get_metrics(seed.id) {
            let mut sim_tracks: Vec<tree::Sim> = Vec::new();

            if byartist == 1 {
                let vals = db.load_artist_tree(&seed.orig_artist);
                let tree = tree::Tree::new(&vals);
                sim_tracks.extend(tree.get_similars(&metrics, NonZero::new(MIN_NUM_SIM).unwrap()));
            } else {
                let tree = req.app_data::<web::Data<tree::Tree>>().unwrap();
                sim_tracks.extend(tree.get_similars(&metrics, NonZero::new(MIN_NUM_SIM).unwrap()));
            }

            for sim_track in sim_tracks {
                let mut trk: Track = get_track_from_id(&db, sim_track.id);
                trk.sim = sim_track.sim;
                if (min > 0 && trk.duration < min) || (max > 0 && trk.duration > max) {
                    log("DISCARD(duration)", &trk);
                    continue;
                }
                if maxbpmdiff > 0 && trk.bpm> 0 && seed.bpm>0 && (trk.bpm-seed.bpm).abs()>maxbpmdiff {
                    log("DISCARD(bpm)", &trk);
                    continue;
                }
                if filter_out_titles.contains(&trk.title) {
                    log("FILTER(title)", &trk);
                    continue;
                }
                if filtergenre == 1 && filter_genre(&trk.genres, &acceptable_genres, &all_genres_from_groups) {
                    log("DISCARD(genre)", &trk);
                    continue;
                }
                if filterxmas == 1 && trk.genres.contains(CHRISTMAS) {
                    log("DISCARD(christmas)", &trk);
                    continue;
                }
                chosen.push(trk.file);
                if chosen.len() >= count {
                    break;
                }
                filter_out_titles.insert(trk.title);
            }
        }
    }

    let mut resp = String::new();
    for track in chosen {
        resp += &track;
        resp += "\n";
    }
    resp
}

pub async fn ready() -> impl Responder {
    "1"
}
