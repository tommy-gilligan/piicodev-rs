#![doc = include_str!("../README.md")]
#![no_std]
#![feature(lint_reasons)]

use core::num::TryFromIntError;
use embedded_hal::i2c::I2c;
use fugit::{ExtU32, Hertz, MillisDuration, RateExtU32};

const REG_STATUS: u8 = 0x01;
const REG_FIRM_MAJ: u8 = 0x02;
const REG_FIRM_MIN: u8 = 0x03;
const REG_TONE: u8 = 0x05;
const REG_LED: u8 = 0x07;
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

impl<I2C: I2c> P18<I2C> {
    pub const fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

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

    pub fn get_led(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c.write_read(self.address, &[REG_LED], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        if on {
            self.i2c.write(self.address, &[REG_LED | 0x80, 1])?;
        } else {
            self.i2c.write(self.address, &[REG_LED | 0x80, 0])?;
        }
        Ok(())
    }

    pub fn firmware(&mut self) -> Result<(u8, u8), I2C::Error> {
        let mut maj_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_FIRM_MAJ], &mut maj_data)?;
        let mut min_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_FIRM_MIN], &mut min_data)?;
        Ok((maj_data[0], min_data[0]))
    }

    // pub fn whoami(&mut self) -> Result<u16, I2C::Error> {
    //     let mut data: [u8; 2] = [0; 2];
    //     self.i2c
    //         .write_read(self.address as u8, &[REG_WHOAMI], &mut data)?;
    //     Ok(u16::from_be_bytes(data))
    // }

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
#[macro_use]
extern crate std;

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal::i2c::ErrorKind;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use fugit::{ExtU32, RateExtU32};

    use crate::{Error, P18};

    #[test]
    pub fn tone() {
        let expectations = [I2cTransaction::write(
            0x5C,
            vec![0x05, 0x02, 0x06, 0x0B, 0xB8],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

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

        let mut p18 = P18::new(i2c, 0x5C);

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

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.no_tone(), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn read_status() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x01], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.read_status(), Ok(1));
        i2c_clone.done();
    }

    #[test]
    pub fn set_led_on() {
        let expectations = [I2cTransaction::write(0x5C, vec![0x87, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.set_led(true), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_led_off() {
        let expectations = [I2cTransaction::write(0x5C, vec![0x87, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.set_led(false), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_off() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x07], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.get_led(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_on() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x07], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.get_led(), Ok(true));
        i2c_clone.done();
    }

    // #[test]
    // pub fn whoami() {
    //     let expectations = [I2cTransaction::write_read(
    //         0x5C,
    //         vec![0x01],
    //         vec![0x01, 0x10],
    //     )];
    //     let i2c = I2cMock::new(&expectations);
    //     let mut i2c_clone = i2c.clone();

    //     let mut p18 = P18::new(i2c, 0x5C);

    //     assert_eq!(p18.whoami(), Ok(0x0110));
    //     i2c_clone.done();
    // }

    #[test]
    pub fn firmware() {
        let expectations = [
            I2cTransaction::write_read(0x5C, vec![0x02], vec![0x01]),
            I2cTransaction::write_read(0x5C, vec![0x03], vec![0x02]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.firmware(), Ok((0x01, 0x02)));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_ok() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x09], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.self_test(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_not_ok() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x09], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18::new(i2c, 0x5C);

        assert_eq!(p18.self_test(), Ok(false));
        i2c_clone.done();
    }
}
