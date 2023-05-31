//! # Unofficial Rust Driver for PiicoDev Color Sensor
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Colour-Sensor-VEML6040/tree/2c2986eafe057aebe93e84157f217c598efd60cf
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-VEML6040-MicroPython-Module/tree/8cb4fc8c2534a9b67a9cae50527892cd902c4b45
//! [Official Product Site]: https://piico.dev/p10
//! [Datasheet]: https://www.vishay.com/docs/84276/veml6040.pdf
use embedded_hal::{delay::DelayUs, i2c::I2c};
use palette::{LinSrgb, SrgbLuma};

const REG_CONF: u8 = 0x00;
const REG_RED: u8 = 0x08;
const REG_GREEN: u8 = 0x09;
const REG_BLUE: u8 = 0x0A;
const REG_WHITE: u8 = 0x0B;
// initialise gain:1x, integration 40ms, Green Sensitivity 0.25168, Max. Detectable Lux 16496
const DEFAULT_SETTINGS: u8 = 0x00;
// No Trig, Auto mode, enabled.
// Disable colour sensor
const SHUTDOWN: u8 = 0x01;

/// Driver for PiicoDev P10
///
/// Typical usage:
///
/// 1. Create an instance through [`P10::new`]
/// 2. Read color information from the instance with [`P10::read`]
pub struct P10<I2C, DELAY> {
    i2c: I2C,
    address: u8,
    delay: DELAY,
}

impl<I2C: I2c, DELAY: DelayUs> P10<I2C, DELAY> {
    /// Acquire a new P10 driver instance
    ///
    /// Arguments:
    /// * `i2c`: should be acquired from the target platform's HAL
    /// * `address`: must match the hardware address of the P10.  This should be 0x10.
    /// * `delay`: should also be acquired from the target platform's HAL
    ///
    /// # Errors
    pub fn new(i2c: I2C, address: u8, delay: DELAY) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            address,
            delay,
        };

        res.i2c.write(res.address, &[REG_CONF, SHUTDOWN])?;
        res.i2c.write(res.address, &[REG_CONF, DEFAULT_SETTINGS])?;
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
            .write_read(self.address, &[REG_RED], &mut data_red)?;
        self.i2c
            .write_read(self.address, &[REG_GREEN], &mut data_green)?;
        self.i2c
            .write_read(self.address, &[REG_BLUE], &mut data_blue)?;
        self.i2c
            .write_read(self.address, &[REG_WHITE], &mut data_white)?;
        let red: u16 = u16::from_le_bytes(data_red);
        let green: u16 = u16::from_le_bytes(data_green);
        let blue: u16 = u16::from_le_bytes(data_blue);
        let white: u16 = u16::from_le_bytes(data_white);

        Ok((LinSrgb::new(red, green, blue), SrgbLuma::new(white)))
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::delay::MockNoop;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use palette::{LinSrgb, SrgbLuma};

    use crate::p10::P10;

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x10, vec![0x00, 0x01]),
            I2cTransaction::write(0x10, vec![0x00, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P10::new(i2c, 0x10, MockNoop {}).unwrap();

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
            address: 0x10,
            delay: MockNoop {},
        };

        assert_eq!(
            p10.read(),
            Ok((LinSrgb::new(0x6812, 0x9021, 0x2190), SrgbLuma::new(0x00AA)))
        );

        i2c_clone.done();
    }
}
