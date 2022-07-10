/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/
use actix_web::{web, HttpRequest, Responder};
use rusqlite::Connection;
use std::fs;
use std::io::Write;
use std::path::Path;

const CHUNK_SIZE: usize = 5 * 1024 * 1024;

pub async fn handle_upload(req: HttpRequest, body: web::Bytes) -> impl Responder {
    let db_path = req.app_data::<web::Data<String>>().unwrap().to_string();
    let path = format!("{}.tmp", db_path);
    let up_path = Path::new(&path);
    let orig_path = Path::new(&db_path);
    let mut total_written = 0;

    if up_path.exists() {
        if let Err(e) = fs::remove_file(up_path) {
            log::info!(
                "Could not remove file {}: {} happened",
                up_path.display(),
                e
            );
        };
    }

    log::info!("Handling upload");
    match fs::File::create(up_path) {
        Ok(mut file) => {
            let mut iter = body.chunks(CHUNK_SIZE);
            for chunk in iter.by_ref() {
                let a = chunk;
                log::info!("Received chunk of {} byte(s)", a.len());
                if let Ok(count) = file.write(a) {
                    total_written += count;
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create temp upload file. {}", e);
        }
    }

    log::debug!("Total size: {}", total_written);
    // Rename DB.tmp to DB - but only if its a valid SQLite database
    if total_written > 0 && up_path.exists() {
        // Ensure file is a valid SQLite database
        match Connection::open(up_path) {
            Ok(conn) => {
                // now close, so that file can be renamed
                match conn.close() {
                    Ok(_) => {
                        // Remove original DB
                        if orig_path.exists() {
                            match fs::remove_file(orig_path) {
                                Ok(_) => {}
                                Err(e) => {
                                    log::error!(
                                        "Failed to remove {}. {}",
                                        orig_path.to_string_lossy(),
                                        e
                                    );
                                }
                            }
                        }

                        // Now do actual rename
                        if !orig_path.exists() {
                            match fs::rename(up_path, orig_path) {
                                Ok(_) => {}
                                Err(e) => {
                                    log::error!(
                                        "Failed to rename {} to {}. {}",
                                        up_path.to_string_lossy(),
                                        orig_path.to_string_lossy(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(_) => {
                        log::error!("Failed to close {}.", up_path.to_string_lossy());
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to open {}. {}", up_path.to_string_lossy(), e);
            }
        }

        // To be safe, remove temp if it still exists
        if up_path.exists() {
            match fs::remove_file(up_path) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to remove {}. {}", up_path.to_string_lossy(), e);
                }
            }
        }
    }
    ""
}
