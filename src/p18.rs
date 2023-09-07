//! # Unofficial Rust Driver for PiicoDev Buzzer
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Buzzer/tree/a3be5160105aa1b62cc5ea01a09b57bd95dbc7fd
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Buzzer-MicroPython-Module/tree/f33f1b08d48f8745377f929bd19472bb967bb36b
//! [Official Product Site]: https://piico.dev/p18
//! [Datasheet]: https://datasheet.lcsc.com/lcsc/1811141116_Jiangsu-Huaneng-Elec-MLT-8540H_C95298.pdf

use crate::Driver;
use core::num::TryFromIntError;
use embedded_hal::i2c::I2c;
use fugit::{ExtU32, Hertz, MillisDuration, RateExtU32};

const REG_STATUS: u8 = 0x01;
const REG_TONE: u8 = 0x05;
const REG_SELF_TEST: u8 = 0x09;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    TryFromIntError(TryFromIntError),
    I2cError(E),
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Self::I2cError(error)
    }
}

pub struct P18<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> Driver<I2C, core::convert::Infallible> for P18<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<I2C: I2c> P18<I2C> {
    pub fn tone(
        &mut self,
        frequency: Hertz<u32>,
        duration: MillisDuration<u32>,
    ) -> Result<(), Error<I2C::Error>> {
        let frequency_bytes: [u8; 2] = u16::to_be_bytes(
            frequency
                .to_Hz()
                .try_into()
                .map_err(Error::TryFromIntError)?,
        );
        let duration_bytes: [u8; 2] = u16::to_be_bytes(
            duration
                .to_millis()
                .try_into()
                .map_err(Error::TryFromIntError)?,
        );
        self.i2c.write(
            self.address,
            &[
                REG_TONE,
                frequency_bytes[0],
                frequency_bytes[1],
                duration_bytes[0],
                duration_bytes[1],
            ],
        )?;
        Ok(())
    }

    pub fn no_tone(&mut self) -> Result<(), Error<I2C::Error>> {
        self.tone(0.Hz(), 0.millis())
    }

    pub fn read_status(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0x00];
        self.i2c
            .write_read(self.address, &[REG_STATUS], &mut data)?;
        Ok(data[0])
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
    use embedded_hal::i2c::ErrorKind;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use fugit::{ExtU32, RateExtU32};

    use crate::p18::{Error, P18};

    #[test]
    pub fn new() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P18::new(i2c, 0x5C).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn tone() {
        let expectations = [I2cTransaction::write(
            0x5C,
            vec![0x05, 0x02, 0x06, 0x0B, 0xB8],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.tone(518_u32.Hz(), 3000_u32.millis()), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn tone_error() {
        let i2c_error = ErrorKind::Other;
        let expectations = [
            I2cTransaction::write(0x5C, vec![0x05, 0x02, 0x06, 0x0B, 0xB8]).with_error(i2c_error),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(
            p18.tone(518_u32.Hz(), 3000_u32.millis()),
            Err(Error::I2cError(i2c_error))
        );
        i2c_clone.done();
    }

    #[test]
    pub fn no_tone() {
        let expectations = [I2cTransaction::write(
            0x5C,
            vec![0x05, 0x00, 0x00, 0x00, 0x00],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.no_tone(), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn read_status() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x01], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.read_status(), Ok(1));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_ok() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x09], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.self_test(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_not_ok() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x09], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.self_test(), Ok(false));
        i2c_clone.done();
    }
}

pub mod atmel;
pub mod whoami;
