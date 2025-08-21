mod db;
mod search;

use rouille::Response;
use serde::Serialize;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::db::Db;
use crate::search::{search_game, search_sync};

const DB_PATH: &str = "./seeds.db";
const MAX_SEEDS: usize = 250_000;
const LISTEN_URL: &str = "127.0.0.1:43277";
const WAIT_DURATION: Duration = Duration::from_millis(1_000);

#[derive(Serialize)]
struct SeedResponse {
    seed: String,
    kind: &'static str,
}

#[derive(Serialize)]
struct StatsResponse {
    game_seeds: usize,
    sync_seeds: usize,
}

fn handle_seed_request(db: &Arc<Db>, is_sync: bool) -> Response {
    loop {
        match db.fetch_and_delete_seed(is_sync) {
            Ok(Some(seed_hex)) => {
                return Response::json(&SeedResponse {
                    seed: seed_hex,
                    kind: if is_sync { "sync" } else { "game" },
                });
            }
            Ok(None) => {
                // No seeds available, wait and retry
                thread::sleep(WAIT_DURATION);
            }
            Err(e) => {
                eprintln!("Database operation failed: {}", e);
                thread::sleep(WAIT_DURATION);
            }
        }
    }
}

fn handle_stats_request(db: &Arc<Db>) -> Response {
    match (db.get_seed_count(false), db.get_seed_count(true)) {
        (Ok(game_seeds), Ok(sync_seeds)) => Response::json(&StatsResponse {
            game_seeds,
            sync_seeds,
        }),
        _ => {
            eprintln!("Failed to get seed counts");
            Response::text("Internal server error").with_status_code(500)
        }
    }
}

fn spawn_search_thread(name: &'static str, db: Arc<Db>, is_sync: bool) {
    thread::spawn(move || {
        println!("Starting {} seed generator thread", name);

        loop {
            let seed = if is_sync {
                search_sync()
            } else {
                search_game()
            };
            let seed_hex = hex::encode(seed);

            loop {
                match db.insert_seed_with_limit(&seed_hex, MAX_SEEDS, is_sync) {
                    Ok(true) => break, // success
                    Ok(false) => {}    // no errors, but limit reached
                    Err(e) => eprintln!("{} generator error inserting seed: {}", name, e),
                }
                // Wait before trying again
                thread::sleep(WAIT_DURATION);
            }
        }
    });
}

fn main() {
    // Initialize the database
    let db = Arc::new(Db::new(DB_PATH));

    println!("Server listening on http://{}", LISTEN_URL);

    // Spawn one thread for game seeds
    spawn_search_thread("game", db.clone(), false);

    // Spawn one thread for sync seeds
    spawn_search_thread("sync", db.clone(), true);

    // Start web server
    rouille::start_server(LISTEN_URL, move |request| {
        match (request.method(), request.url().as_str()) {
            ("POST", "/seed/game") => handle_seed_request(&db, false),
            ("POST", "/seed/sync") => handle_seed_request(&db, true),
            ("GET", "/stats") => handle_stats_request(&db),
            _ => Response::empty_404(),
        }
    });
}
