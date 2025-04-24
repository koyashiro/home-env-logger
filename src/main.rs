mod db;
mod measurement;
mod mh_z19c;
mod sensor;

use std::time::Duration;

use anyhow::Context;
use backon::{BlockingRetryable, ConstantBuilder};
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

use crate::{db::DB, sensor::Sensor};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_time_offset_to_local()
            .map_err(|_| anyhow::anyhow!("Failed to set time offset to local"))?
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")?;

    if let Err(e) = run().await {
        log::error!("{e}");
    }

    Ok(())
}

pub async fn run() -> Result<(), anyhow::Error> {
    let retry_builder = ConstantBuilder::default()
        .with_delay(Duration::from_millis(100))
        .with_max_times(20);

    let mut sensor = Sensor::new().context("Failed to initialize sensor")?;
    (|| sensor.init())
        .retry(retry_builder)
        .notify(|e, dur| {
            log::error!("{e}");
            log::info!("Retrying in {:?}", dur);
        })
        .call()?;

    let db = DB::new().context("Failed to initialize database")?;
    db.init().context("Failed to initialize database")?;

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;

            let measurement = match (|| sensor.measure())
                .retry(retry_builder)
                .notify(|e, dur| {
                    log::error!("{e}");
                    log::info!("Retrying in {:?}", dur);
                })
                .call()
            {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to read sensor data: {}", e);
                    continue;
                }
            };

            if let Err(e) = db.insert(&measurement) {
                log::error!("Failed to insert data into database: {e}");
                continue;
            }

            log::info!("{measurement:?}");
        }
    });

    tokio::signal::ctrl_c()
        .await
        .context("Failed to wait for Ctrl+C signal")?;

    Ok(())
}
