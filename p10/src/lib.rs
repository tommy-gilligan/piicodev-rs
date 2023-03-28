#![no_std]

use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::I2c;
use palette::{LinSrgb, SrgbLuma};

#[derive(Copy, Clone)]
pub enum Address {
    X10 = 0x10,
}

pub struct P10<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
    address: Address,
}

const CONF: u8 = 0x00;
const REG_RED: u8 = 0x08;
const REG_GREEN: u8 = 0x09;
const REG_BLUE: u8 = 0x0A;
const REG_WHITE: u8 = 0x0B;

// initialise gain:1x, integration 40ms, Green Sensitivity 0.25168, Max. Detectable Lux 16496
const DEFAULT_SETTINGS: u8 = 0x00;

// No Trig, Auto mode, enabled.
// Disable colour sensor
const SHUTDOWN: u8 = 0x01;

impl<I2C: I2c, DELAY: DelayUs> P10<I2C, DELAY> {
    /// # Errors
    pub fn new(i2c: I2C, address: Address, delay: DELAY) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            delay,
            address,
        };

        res.i2c.write(res.address as u8, &[CONF, SHUTDOWN])?;
        res.i2c
            .write(res.address as u8, &[CONF, DEFAULT_SETTINGS])?;
        res.delay.delay_ms(50);

        Ok(res)
    }

    /// # Errors
    pub fn read(&mut self) -> Result<(LinSrgb<u16>, SrgbLuma<u16>), I2C::Error> {
        let mut data_red: [u8; 2] = [0; 2];
        let mut data_green: [u8; 2] = [0; 2];
        let mut data_blue: [u8; 2] = [0; 2];
        let mut data_white: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address as u8, &[REG_RED], &mut data_red)?;
        self.i2c
            .write_read(self.address as u8, &[REG_GREEN], &mut data_green)?;
        self.i2c
            .write_read(self.address as u8, &[REG_BLUE], &mut data_blue)?;
        self.i2c
            .write_read(self.address as u8, &[REG_WHITE], &mut data_white)?;
        let red: u16 = u16::from_le_bytes(data_red);
        let green: u16 = u16::from_le_bytes(data_green);
        let blue: u16 = u16::from_le_bytes(data_blue);
        let white: u16 = u16::from_le_bytes(data_white);

        Ok((LinSrgb::new(red, green, blue), SrgbLuma::new(white)))
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
#[macro_use]
extern crate std;

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::delay::MockNoop;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use palette::{LinSrgb, SrgbLuma};

    use crate::{Address, P10};

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x10, vec![0x00, 0x01]),
            I2cTransaction::write(0x10, vec![0x00, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P10::new(i2c, Address::X10, MockNoop {}).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [
            I2cTransaction::write_read(0x10, vec![0x08], vec![0x12, 0x68]),
            I2cTransaction::write_read(0x10, vec![0x09], vec![0x21, 0x90]),
            I2cTransaction::write_read(0x10, vec![0x0A], vec![0x90, 0x21]),
            I2cTransaction::write_read(0x10, vec![0x0B], vec![0xAA, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p10 = P10 {
            i2c,
            address: Address::X10,
            delay: MockNoop {},
        };

        assert_eq!(
            p10.read(),
            Ok((LinSrgb::new(0x6812, 0x9021, 0x2190), SrgbLuma::new(0x00AA)))
        );

        i2c_clone.done();
    }
}
