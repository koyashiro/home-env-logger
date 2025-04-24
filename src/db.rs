use anyhow::Context;
use rusqlite::Connection;

use crate::measurement::Measurement;

pub const DB_FILE: &str = "./home-env-log.db";

#[derive(Debug)]
pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new() -> Result<Self, anyhow::Error> {
        let conn = Connection::open(DB_FILE).context("Failed to open database file")?;
        Ok(Self { conn })
    }

    pub fn init(&self) -> Result<(), anyhow::Error> {
        self.conn
            .execute_batch(
                r"
                CREATE TABLE IF NOT EXISTS measurements (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL,
                    temperature REAL NOT NULL,
                    humidity REAL NOT NULL,
                    pressure REAL NOT NULL,
                    co2_concentration INTEGER NOT NULL
                );
                ",
            )
            .context("Failed to create table")?;
        Ok(())
    }

    pub fn insert(&self, data: &Measurement) -> Result<(), anyhow::Error> {
        self.conn
            .execute(
                r"
                INSERT INTO measurements (timestamp, temperature, humidity, pressure, co2_concentration) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    data.timestamp.to_rfc3339(),
                    data.temperature,
                    data.humidity,
                    data.pressure,
                    data.co2_concentration,
                ],
            )
            .context("Failed to insert data into table")?;
        Ok(())
    }
}
