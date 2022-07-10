/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use actix_web::{client, web, App, HttpServer};
use argparse::{ArgumentParser, Store, StoreTrue};
use std::path::Path;
use std::process;
mod api;
mod db;
mod tree;
mod upload;

const VERSION: &str = env!("CARGO_PKG_VERSION");

async fn send_port_to_lms(lms_server: &String, port: u16) {
    if !lms_server.is_empty() {
        // Inform LMS of port number in use
        let client = client::Client::default();

        let request = serde_json::json!({
            "id": 1,
            "method": "slim.request",
            "params":[
                "",
                ["blissmixer", "port", format!("number:{}", port)]
            ]
        });

        match client.post(format!("http://{}:9000/jsonrpc.js", lms_server)).send_json(&request).await {
            Ok(_) => {
                log::debug!("LMS updated");
            }
            Err(e) => {
                log::error!("Failed to update LMS. {}", e);
                process::exit(-1);
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut db_path = "bliss.db".to_string();
    let mut port: u16 = 12000;
    let mut address = "0.0.0.0".to_string();
    let mut logging = "warn".to_string();
    let mut lms_server = String::new();
    let mut allow_db_upload = false;
    {
        let db_path_help = format!("Database location (default: {})", db_path);
        let port_help = format!("Port number (default: {})", port);
        let address_help = format!("Address to use (default: {})", address);
        let description = format!("Bliss Mixer v{}", VERSION);

        // arg_parse.refer 'borrows' db_path, etc, and can only have one
        // borrow per scope, hence this section is enclosed in { }
        let mut arg_parse = ArgumentParser::new();
        arg_parse.set_description(&description);
        arg_parse.refer(&mut db_path).add_option(&["-d", "--db"], Store, &db_path_help);
        arg_parse.refer(&mut port).add_option(&["-p", "--port"], Store, &port_help);
        arg_parse.refer(&mut address).add_option(&["-a", "--address"], Store, &address_help);
        arg_parse.refer(&mut logging).add_option(&["-l", "--logging"], Store, "Log level (trace, debug, info, warn, error)");
        arg_parse.refer(&mut lms_server).add_option(&["-L", "--lms"], Store, "LMS server (hostname or IP address)");
        arg_parse.refer(&mut allow_db_upload).add_option(&["-u", "--upload"], StoreTrue, "Allow uploading of database");
        arg_parse.parse_args_or_exit();
    }

    if logging.eq_ignore_ascii_case("trace") || logging.eq_ignore_ascii_case("debug") || logging.eq_ignore_ascii_case("info")
        || logging.eq_ignore_ascii_case("warn") || logging.eq_ignore_ascii_case("error") {
        env_logger::init_from_env(env_logger::Env::default().filter_or("XXXXXXXX", logging));
    } else {
        env_logger::init_from_env(env_logger::Env::default().filter_or("XXXXXXXX", "ERROR"));
        log::error!("Invalid log level ({}) supplied", logging);
        process::exit(-1);
    }

    if db_path.len() < 3 {
        log::error!("Invalid DB path ({}) supplied", db_path);
        process::exit(-1);
    }

    let path = Path::new(&db_path);
    if !allow_db_upload {
        // DB upload not allowd, so database file *must* exist
        if !path.exists() {
            log::error!("DB path ({}) does not exist", db_path);
            process::exit(-1);
        }

        if !path.is_file() {
            log::error!("DB path ({}) is not a file", db_path);
            process::exit(-1);
        }
    }

    if !lms_server.is_empty() {
        port = 0;
    }

    if allow_db_upload {
        log::info!("Starting in upload mode");
        let server = HttpServer::new(move || {
            App::new()
                .data(db_path.clone())
                .app_data(web::PayloadConfig::new(200 * 1024 * 1024))
                .route("/upload", web::put().to(upload::handle_upload))
        }).bind((address, port))?;
        send_port_to_lms(&lms_server, server.addrs()[0].port()).await;

        server.run().await
    } else {
        log::info!("Starting in mix mode");
        let mut tree = tree::Tree::new();
        if path.exists() {
            let db = db::Db::new(&db_path);
            db.load_tree(&mut tree);
            db.close();
        }

        let server = HttpServer::new(move || {
            App::new()
                .data(tree.clone())
                .data(db_path.clone())
                .route("/api/mix", web::post().to(api::mix))
                .route("/api/list", web::post().to(api::list))
                .route("/api/ready", web::get().to(api::ready))
        }).bind((address, port))?;
        send_port_to_lms(&lms_server, server.addrs()[0].port()).await;

        server.run().await
    }
}
