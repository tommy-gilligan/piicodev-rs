//! # Unofficial Rust Driver for `PiicoDev` Ultrasonic Rangefinder
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Ultrasonic-Rangefinder/tree/3e006745fdc5123f8bc55bbcfde54461db72883c
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Ultrasonic-Rangefinder-MicroPython-Module/tree/6f5f4a65f41b29be7fd7c041cde15104fed2f31c
//! [Official Product Site]: https://piico.dev/p30
//! [Datasheet]: https://cdn.sparkfun.com/datasheets/Sensors/Proximity/HCSR04.pdf

use crate::Driver;
use embedded_hal::i2c::I2c;

const REG_RAW: u8 = 0x05;
const REG_PERIOD: u8 = 0x06;
const REG_STATUS: u8 = 0x08;
const REG_SELF_TEST: u8 = 0x09;

pub struct P30<I2C> {
    i2c: I2C,
    address: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    I2cError(E),
    ArgumentError,
    UnexpectedDevice,
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Self::I2cError(error)
    }
}

impl<I2C: I2c> Driver<I2C, Error<I2C::Error>> for P30<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    fn init_inner(mut self) -> Result<Self, Error<I2C::Error>> {
        self.set_period(20)?;

        Ok(self)
    }
}

impl<I2C: I2c> P30<I2C> {
    pub fn ready(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_STATUS], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn read(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c.write_read(self.address, &[REG_RAW], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn get_period(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_PERIOD], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn set_period(&mut self, period: u16) -> Result<(), I2C::Error> {
        let bytes: [u8; 2] = u16::to_be_bytes(period);
        self.i2c
            .write(self.address, &[REG_PERIOD | 0x80, bytes[0], bytes[1]])?;
        Ok(())
    }

    pub fn self_test(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_SELF_TEST], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use crate::Driver;
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;

    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p30::P30;

    #[test]
    pub fn self_test_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 { i2c, address: 0x35 };

        assert_eq!(p30.self_test(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_not_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 { i2c, address: 0x35 };

        assert_eq!(p30.self_test(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn ready() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x08], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 { i2c, address: 0x35 };

        assert_eq!(p30.ready(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn new_sample_unavailable() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x08], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 { i2c, address: 0x35 };

        assert_eq!(p30.ready(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x05],
            vec![0x9B, 0x2B],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 { i2c, address: 0x35 };

        assert_eq!(p30.read(), Ok(39723));
        i2c_clone.done();
    }

    #[test]
    pub fn set_period() {
        let expectations = [I2cTransaction::write(0x35, vec![0x86, 0x07, 0xD0])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 { i2c, address: 0x35 };

        assert_eq!(p30.set_period(2000), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn get_period() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x06],
            vec![0x03, 0xE8],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 { i2c, address: 0x35 };

        assert_eq!(p30.get_period(), Ok(1000));
        i2c_clone.done();
    }

    #[test]
    pub fn new() {
        let expectations = [I2cTransaction::write(0x35, vec![0x86, 0x00, 0x14])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P30::new(i2c, 0x35).unwrap().init().unwrap();

        i2c_clone.done();
    }
}

pub mod atmel;
pub mod helper;
pub mod whoami;
