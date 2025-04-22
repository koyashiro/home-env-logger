mod mh_z19c;

use std::thread;
use std::time::Duration;

use anyhow::Context;
use bme280::i2c::BME280;
use chrono::{Local, SecondsFormat};
use rppal::hal::Delay;
use rppal::i2c::I2c;

use crate::mh_z19c::MHZ19C;

fn main() -> Result<(), anyhow::Error> {
    let i2c = I2c::new().context("Failed to initialize I2C")?;
    let mut bme280 = BME280::new_primary(i2c);
    let mut delay = Delay;
    bme280
        .init(&mut delay)
        .context("Failed to initialize BME280")?;

    let mut mhz19c = MHZ19C::new().context("Failed to initialize MH-Z19C")?;
    mhz19c.init().context("Failed to initialize MH-Z19C")?;

    loop {
        let measurements = match bme280.measure(&mut delay) {
            Ok(measurements) => measurements,
            Err(e) => {
                eprintln!("Failed to read BME280: {}", e);
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let temperature = (measurements.temperature * 10f32).round() / 10f32;
        let humidity = (measurements.humidity * 10f32).round() / 10f32;
        let pressure = ((measurements.pressure / 100f32) * 10f32).round() / 10f32;

        let co2_concentration = match mhz19c.read_co2_concentration() {
            Ok(co2_concentration) => co2_concentration,
            Err(e) => {
                eprintln!("Failed to read MH-Z19C: {}", e);
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };
        // .context("Failed to read CO2 concentration")?;

        println!(
            "[{}] temperature: {:.1} degC, humidity: {:.1} %, pressure: {:.1} hPa, co2: {} ppm",
            Local::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            temperature,
            humidity,
            pressure,
            co2_concentration
        );

        thread::sleep(Duration::from_secs(1));
    }
}
