use anyhow::Context;
use rppal::uart::{Parity, Uart};

pub const BAUD_RATE: u32 = 9600;
pub const PARITY: Parity = Parity::None;
pub const DATA_BITS: u8 = 8;
pub const STOP_BITS: u8 = 1;

pub const READ_COMMAND: [u8; 9] = [0xff, 0x01, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79];

pub const RETURN_VALUE_START_BYTE: u8 = 0xff;
pub const RETURN_VALUE_COMMAND: u8 = 0x86;

#[derive(Debug)]
pub struct MHZ19C {
    uart: Uart,
}

impl MHZ19C {
    pub fn new() -> Result<MHZ19C, anyhow::Error> {
        let uart = Uart::new(BAUD_RATE, PARITY, DATA_BITS, STOP_BITS)
            .context("Failed to initialize UART")?;

        Ok(MHZ19C { uart })
    }

    pub fn init(&mut self) -> Result<(), anyhow::Error> {
        self.uart
            .set_read_mode(9, std::time::Duration::from_millis(10))
            .context("Failed to set read mode")?;

        Ok(())
    }

    pub fn read_co2_concentration(&mut self) -> Result<u16, anyhow::Error> {
        self.uart
            .write(&READ_COMMAND)
            .context("Failed to write command to UART")?;

        let mut response = [0u8; 9];
        self.uart
            .read(&mut response)
            .context("Failed to read response from UART")?;

        if response[0] != RETURN_VALUE_START_BYTE || response[1] != RETURN_VALUE_COMMAND {
            return Err(anyhow::anyhow!(
                "Invalid response: expected start byte {} and command {}, got {} and {}",
                RETURN_VALUE_START_BYTE,
                RETURN_VALUE_COMMAND,
                response[0],
                response[1]
            ));
        }

        let checksum = calculate_checksum(&response);
        if response[8] != checksum {
            return Err(anyhow::anyhow!(
                "Invalid checksum: expected {}, got {}",
                checksum,
                response[8]
            ));
        }

        let concentration = ((response[2] as u16) << 8) | response[3] as u16;

        Ok(concentration)
    }
}

pub fn calculate_checksum(data: &[u8; 9]) -> u8 {
    let mut checksum: u8 = 0;
    for &packet in data.iter().skip(1).take(7) {
        checksum = checksum.wrapping_add(packet);
    }
    checksum = 0xff - checksum;
    checksum = checksum.wrapping_add(1);
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_checksum() {
        let data = &READ_COMMAND;
        let checksum = calculate_checksum(data);
        assert_eq!(checksum, data[8]);
    }
}
