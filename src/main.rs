mod search;

use once_cell::sync::Lazy;
use rouille::Response;
use rusqlite::Connection;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const DB_PATH: &str = "./seeds.db";
const MAX_SEEDS: usize = 250_000;
const LISTEN_URL: &str = "127.0.0.1:6666";

type DbResult<T> = Result<T, Box<dyn std::error::Error>>;

// Global database connection using once_cell
static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let db = Connection::open(DB_PATH).unwrap();
    db.execute(
        "CREATE TABLE IF NOT EXISTS seeds (seed TEXT PRIMARY KEY)",
        [],
    )
    .unwrap();
    Mutex::new(db)
});

// Get the total seeds
fn get_seed_count() -> DbResult<usize> {
    let db = DB.lock().map_err(|e| format!("Failed to lock DB: {}", e))?;
    let count: usize = db.query_row("SELECT COUNT(*) FROM seeds", [], |row| row.get(0))?;
    Ok(count)
}

// Insert the seed into the db
fn insert_seed(seed_hex: &str) -> DbResult<()> {
    let db = DB.lock().map_err(|e| format!("Failed to lock DB: {}", e))?;
    db.execute("INSERT OR IGNORE INTO seeds VALUES (?)", [seed_hex])?;
    Ok(())
}

// Tries to insert the seed, returning `true` if successful and `false` if the limit is reached
fn insert_seed_with_limit(seed_hex: &str) -> DbResult<bool> {
    let seed_count = get_seed_count()?;
    if seed_count >= MAX_SEEDS {
        return Ok(false);
    }
    insert_seed(seed_hex)?;
    Ok(true)
}

// Get a seed from the db without replacement, if one exists
fn fetch_and_delete_seed() -> DbResult<Option<String>> {
    let mut db = DB.lock().map_err(|e| format!("Failed to lock DB: {}", e))?;
    let tx = db.transaction()?;

    let seed_hex: Option<String> =
        tx.query_row("SELECT seed FROM seeds LIMIT 1", [], |row| row.get(0))?;

    if let Some(ref seed) = seed_hex {
        tx.execute("DELETE FROM seeds WHERE seed = ?", [seed])?;
    }

    tx.commit()?;
    Ok(seed_hex)
}

fn handle_seed_request() -> Response {
    loop {
        match fetch_and_delete_seed() {
            Ok(Some(seed_hex)) => {
                return Response::json(&serde_json::json!({
                    "seed": seed_hex
                }));
            }
            Ok(None) => {
                // No seeds available, wait and retry
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                eprintln!("Database operation failed: {}", e);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn main() {
    // Calculate generator threads (1/3 of system threads)
    let generator_threads = std::thread::available_parallelism()
        .map(|n| n.get() / 3)
        .unwrap_or(1)
        .max(1);

    // Spawn seed generators
    for thread_id in 0..generator_threads {
        thread::spawn(move || {
            loop {
                let seed = search::search();
                let seed_hex = hex::encode(seed);

                loop {
                    match insert_seed_with_limit(&seed_hex) {
                        Ok(true) => break, // success
                        Ok(false) => {}    // no errors, but limit reached
                        Err(e) => eprintln!("thread {} had error inserting seed: {}", thread_id, e),
                    }
                    // Wait before trying again
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
    }

    println!(
        "Server listening on http://{} with {} generator threads",
        LISTEN_URL, generator_threads
    );

    // Start web server
    rouille::start_server(LISTEN_URL, |request| {
        match (request.method(), request.url().as_str()) {
            ("POST", "/seed") => handle_seed_request(),
            _ => Response::empty_404(),
        }
    });
}
