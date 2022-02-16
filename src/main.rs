/**
 * BlissMixer: Use Bliss analysis results to create music mixes
 *
 * Copyright (c) 2022 Craig Drummond <craig.p.drummond@gmail.com>
 * GPLv3 license.
 *
 **/

use actix_web::{App, HttpServer, web};
use argparse::{ArgumentParser, Store};
use std::path::Path;
use std::process;
mod api;
mod db;
mod tree;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut db_path = "bliss.db".to_string();
    let mut port:u16 = 12000;
    let mut address = "0.0.0.0".to_string();
    let mut logging = "warn".to_string();
    {
        let db_path_help = format!("Database location (default: {})", db_path);
        let port_help = format!("Port number (default: {})", port);
        let address_help = format!("Address to use (default: {})", address);
        // arg_parse.refer 'borrows' db_path, etc, and can only have one
        // borrow per scope, hence this section is enclosed in { }
        let mut arg_parse = ArgumentParser::new();
        arg_parse.set_description("Bliss Mixer");
        arg_parse.refer(&mut db_path).add_option(&["-d", "--db"], Store, &db_path_help);
        arg_parse.refer(&mut port).add_option(&["-p", "--port"], Store, &port_help);
        arg_parse.refer(&mut address).add_option(&["-a", "--address"], Store, &address_help);
        arg_parse.refer(&mut logging).add_option(&["-l", "--logging"], Store, "Log level (trace, debug, info, warn, error)");
        arg_parse.parse_args_or_exit();
    }

    if logging.eq_ignore_ascii_case("trace") || logging.eq_ignore_ascii_case("debug") || logging.eq_ignore_ascii_case("info") || logging.eq_ignore_ascii_case("warn") || logging.eq_ignore_ascii_case("error") {
        env_logger::init_from_env(env_logger::Env::default().filter_or("XXXXXXXX", logging));
    } else {
        env_logger::init_from_env(env_logger::Env::default().filter_or("XXXXXXXX", "ERROR"));
        log::error!("Invalid log level ({}) supplied", logging);
        process::exit(-1); 
    }

    if db_path.len()<3 {
        log::error!("Invalid DB path ({}) supplied", db_path);
        process::exit(-1);
    }

    let path = Path::new(&db_path);
    if !path.exists() {
        log::error!("DB path ({}) does not exist", db_path);
        process::exit(-1);
    }

    if !path.is_file() {
        log::error!("DB path ({}) is not a file", db_path);
        process::exit(-1);
    }

    let mut tree = tree::Tree::new();
    let db = db::Db::new(&db_path);
    db.load_tree(&mut tree);
    db.close();
        
    HttpServer::new(move|| {
        App::new()
            .data(tree.clone())
            .data(db_path.clone())
            .route("/api/similar", web::post().to(api::similar))
    })
    .bind((address, port))?
    .run()
    .await
}
