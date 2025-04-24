use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct Measurement {
    pub timestamp: DateTime<Local>,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub co2_concentration: u16,
}
