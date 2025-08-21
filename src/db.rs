use rusqlite::{Connection, OptionalExtension};
use std::sync::Mutex;

pub type DbResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn new(db_path: &str) -> Self {
        let conn = Connection::open(db_path).unwrap();

        // Migrate old seeds table to game_seeds if it exists
        let _ = conn.execute("ALTER TABLE seeds RENAME TO game_seeds", []); // Ignore error if table doesn't exist

        // Create tables for both game and sync seeds
        conn.execute(
            "CREATE TABLE IF NOT EXISTS game_seeds (seed TEXT PRIMARY KEY)",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_seeds (seed TEXT PRIMARY KEY)",
            [],
        )
        .unwrap();

        Self {
            conn: Mutex::new(conn),
        }
    }

    fn get_table_name(is_sync: bool) -> &'static str {
        if is_sync { "sync_seeds" } else { "game_seeds" }
    }

    // Get the total seeds in the specified table
    pub fn get_seed_count(&self, is_sync: bool) -> DbResult<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Failed to lock DB: {}", e))?;
        let table = Self::get_table_name(is_sync);
        let query = format!("SELECT COUNT(*) FROM {}", table);
        let count: usize = conn.query_row(&query, [], |row| row.get(0))?;
        Ok(count)
    }

    // Insert the seed into the specified table
    pub fn insert_seed(&self, seed_hex: &str, is_sync: bool) -> DbResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Failed to lock DB: {}", e))?;
        let table = Self::get_table_name(is_sync);
        let query = format!("INSERT OR IGNORE INTO {} VALUES (?)", table);
        conn.execute(&query, [seed_hex])?;
        Ok(())
    }

    // Tries to insert the seed, returning `true` if successful and `false` if the limit is reached
    pub fn insert_seed_with_limit(
        &self,
        seed_hex: &str,
        max_seeds: usize,
        is_sync: bool,
    ) -> DbResult<bool> {
        let seed_count = self.get_seed_count(is_sync)?;
        if seed_count >= max_seeds {
            return Ok(false);
        }
        self.insert_seed(seed_hex, is_sync)?;
        Ok(true)
    }

    // Get a seed from the specified table without replacement, if one exists
    pub fn fetch_and_delete_seed(&self, is_sync: bool) -> DbResult<Option<String>> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| format!("Failed to lock DB: {}", e))?;
        let tx = conn.transaction()?;
        let table = Self::get_table_name(is_sync);

        let query = format!("SELECT seed FROM {} LIMIT 1", table);
        let seed_hex: Option<String> = tx.query_row(&query, [], |row| row.get(0)).optional()?;

        if let Some(ref seed) = seed_hex {
            let delete_query = format!("DELETE FROM {} WHERE seed = ?", table);
            tx.execute(&delete_query, [seed])?;
        }

        tx.commit()?;
        Ok(seed_hex)
    }
}
