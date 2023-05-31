//! # Unofficial Rust Driver for PiicoDev 3x RGB LED
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-3x-RGB-LED-Module/tree/afd2e878f9389ce49cdacc8e39a382eb24dcc957
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-RGB-LED-MicroPython-Module/tree/59b4821f561e38030c29bf9a7df5af6350980e76
//! [Official Product Site]: https://piico.dev/p13
//! [Datasheet]: https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf
use core::num::TryFromIntError;
use embedded_hal::i2c::I2c;
use smart_leds_trait::SmartLedsWrite;

pub struct P13<I2C> {
    i2c: I2C,
    address: u8,
}

const REG_WHOAMI: u8 = 0x00;
const REG_FIRM_MIN: u8 = 0x01;
const REG_FIRM_MAJ: u8 = 0x02;
const REG_LED: u8 = 0x03;
const REG_I2C_ADDRESS: u8 = 0x05;

const DEVICE_ID: u8 = 0x84;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    TryFromIntError(TryFromIntError),
    I2cError(E),
    ArgumentError,
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Self::I2cError(error)
    }
}

impl<I2C: I2c> P13<I2C> {
    pub const fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// # Errors
    pub fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        if on {
            self.i2c.write(self.address, &[REG_LED, 1])?;
        } else {
            self.i2c.write(self.address, &[REG_LED, 0])?;
        }
        Ok(())
    }

    // 0x0084 132
    /// # Errors
    pub fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_WHOAMI], &mut data)?;
        Ok(u16::from_be_bytes([0, data[0]]))
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

    // should somehow destroy instance after call
    pub fn set_address(&mut self, new_address: u8) -> Result<(), Error<I2C::Error>> {
        if !(0x08..=0x77).contains(&new_address) {
            return Err(Error::ArgumentError);
        }
        self.i2c
            .write(self.address, &[REG_I2C_ADDRESS, new_address])?;
        Ok(())
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
}

impl<I2C: I2c> SmartLedsWrite for P13<I2C> {
    type Color = smart_leds_trait::RGB8;
    type Error = I2C::Error;

    /// # Errors
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        let mut data: [u8; 10] = [0x07, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];
        for (i, v) in iterator
            .flat_map(|item| {
                let color: Self::Color = item.into();
                [color.r, color.g, color.b]
            })
            .enumerate()
        {
            data[i + 1] = v;
        }
        self.i2c.write(self.address, &data)?;

        Ok(())
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use smart_leds_trait::{SmartLedsWrite, RGB};

    use crate::p13::{Error, P13};

    #[test]
    pub fn write() {
        let expectations = [I2cTransaction::write(
            0x0A,
            vec![0x07, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p13 = P13::new(i2c, 0x0A);

        let data: [RGB<u8>; 3] = [
            RGB {
                r: 0xff,
                g: 0x00,
                b: 0x00,
            },
            RGB {
                r: 0xff,
                g: 0xff,
                b: 0x00,
            },
            RGB {
                r: 0xff,
                g: 0x00,
                b: 0xff,
            },
        ];
        p13.write(data.iter().copied()).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn set_address() {
        let expectations = [I2cTransaction::write(0x09, vec![0x05, 0x69])];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p13 = P13 { i2c, address: 0x09 };
        p13.set_address(0x69).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_small() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p13 = P13 { i2c, address: 0x09 };
        assert_eq!(p13.set_address(0x07), Err(Error::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_large() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p13 = P13 { i2c, address: 0x09 };
        assert_eq!(p13.set_address(0x78), Err(Error::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x00], vec![2])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p13 = P13 { i2c, address: 0x09 };
        assert_eq!(p13.whoami(), Ok(2));

        i2c_clone.done();
    }

    #[test]
    pub fn set_led_on() {
        let expectations = [I2cTransaction::write(0x09, vec![0x03, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p13 = P13 { i2c, address: 0x09 };

        assert_eq!(p13.set_led(true), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_led_off() {
        let expectations = [I2cTransaction::write(0x09, vec![0x03, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p13 = P13 { i2c, address: 0x09 };

        assert_eq!(p13.set_led(false), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_off() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x03], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p13 = P13 { i2c, address: 0x09 };

        assert_eq!(p13.get_led(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_on() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x03], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p13 = P13 { i2c, address: 0x09 };

        assert_eq!(p13.get_led(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn firmware() {
        let expectations = [
            I2cTransaction::write_read(0x09, vec![0x02], vec![0x03]),
            I2cTransaction::write_read(0x09, vec![0x01], vec![0x02]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p13 = P13 { i2c, address: 0x09 };

        assert_eq!(p13.firmware(), Ok((0x03, 0x02)));
        i2c_clone.done();
    }
}
