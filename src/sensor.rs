use anyhow::Context as _;
use bme280::i2c::BME280;
use chrono::Local;
use rppal::{hal::Delay, i2c::I2c};

use crate::{measurement::Measurement, mh_z19c::MHZ19C};

#[derive(Debug)]
pub struct Sensor {
    delay: Delay,
    bme280: BME280<I2c>,
    hz19c: MHZ19C,
}

impl Sensor {
    pub fn new() -> Result<Sensor, anyhow::Error> {
        let delay = Delay;
        let i2c = I2c::new().context("Failed to initialize I2C")?;
        let bme280 = BME280::new_primary(i2c);
        let hz19c = MHZ19C::new().context("Failed to initialize MH-Z19C")?;

        Ok(Sensor {
            delay,
            bme280,
            hz19c,
        })
    }

    pub fn init(&mut self) -> Result<(), anyhow::Error> {
        self.bme280
            .init(&mut self.delay)
            .context("Failed to initialize BME280")?;
        self.hz19c.init().context("Failed to initialize MH-Z19C")?;

        Ok(())
    }

    pub fn measure(&mut self) -> Result<Measurement, anyhow::Error> {
        let co2_concentration = self.hz19c.read_co2_concentration()?;
        let m = self.bme280.measure(&mut self.delay)?;

        Ok(Measurement {
            timestamp: Local::now(),
            temperature: m.temperature,
            humidity: m.humidity,
            pressure: m.pressure,
            co2_concentration,
        })
    }
}
