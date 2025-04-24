use std::time::Duration;

use anyhow::Context;
use backon::BlockingRetryable;
use backon::ConstantBuilder;
use bme280::i2c::BME280;
use chrono::Local;
use db::DB;
use log::LevelFilter;
use measurement::Measurement;
use mh_z19c::MHZ19C;
use rppal::{hal::Delay, i2c::I2c};
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

mod db;
mod measurement;
mod mh_z19c;

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

    let mut bme280 = BME280::new_primary(I2c::new().context("Failed to initialize I2C")?);
    (|| bme280.init(&mut Delay))
        .retry(retry_builder)
        .notify(|e, dur| {
            log::error!("{e}");
            log::info!("Retrying in {:?}", dur);
        })
        .call()?;

    let mut delay = Delay;

    let mut mhz19c = MHZ19C::new().context("Failed to initialize MH-Z19C")?;
    (|| mhz19c.init())
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

            let measurements = match (|| bme280.measure(&mut delay))
                .retry(retry_builder)
                .notify(|e, dur| {
                    log::error!("{e}");
                    log::info!("Retrying in {:?}", dur);
                })
                .call()
            {
                Ok(measurements) => measurements,
                Err(e) => {
                    log::error!("Failed to read BME280 measurements: {}", e);
                    continue;
                }
            };

            let co2_concentration = match mhz19c.read_co2_concentration() {
                Ok(co2_concentration) => co2_concentration,
                Err(e) => {
                    log::error!("Failed to read MH-Z19C CO2 concentration: {}", e);
                    continue;
                }
            };

            let data = Measurement {
                timestamp: Local::now(),
                temperature: measurements.temperature,
                humidity: measurements.humidity,
                pressure: measurements.pressure,
                co2_concentration,
            };

            if let Err(e) = db.insert(&data) {
                log::error!("Failed to insert data into database: {e}");
                continue;
            }

            log::info!("{data:?}");
        }
    });

    tokio::signal::ctrl_c()
        .await
        .context("Failed to wait for Ctrl+C signal")?;

    Ok(())
}
