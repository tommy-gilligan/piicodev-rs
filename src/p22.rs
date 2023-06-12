//! # Unofficial Rust Driver for PiicoDev Potentiometer
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Slide-Potentiometer/tree/33bdc7dce717f466197d7363b005aaf69f7caac6
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Potentiometer-MicroPython-Module/tree/bdb408159cab040e7d374f5254e4d4700088422e
//! [Official Product Site]: https://piico.dev/p22
use embedded_hal::i2c::I2c;

const REG_WHOAMI: u8 = 0x01;
const REG_FIRM_MAJ: u8 = 0x02;
const REG_FIRM_MIN: u8 = 0x03;
const REG_I2C_ADDRESS: u8 = 0x04;
const REG_POT: u8 = 0x05;
const REG_LED: u8 = 0x07;
const REG_SELF_TEST: u8 = 0x09;

pub struct P22<I2C> {
    i2c: I2C,
    address: u8,
}

use crate::Driver;
impl<I2C: I2c> Driver<I2C> for P22<I2C> {
    fn alloc(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<I2C: I2c> P22<I2C> {
    pub fn read(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c.write_read(self.address, &[REG_POT], &mut data)?;
        Ok(u16::from_be_bytes(data))
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

use crate::{Atmel, SetAddressError};
impl<I2C: I2c> Atmel<I2C> for P22<I2C> {
    fn get_led(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c.write_read(self.address, &[REG_LED], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        if on {
            self.i2c.write(self.address, &[REG_LED | 0x80, 1])?;
        } else {
            self.i2c.write(self.address, &[REG_LED | 0x80, 0])?;
        }
        Ok(())
    }

    fn firmware(&mut self) -> Result<(u8, u8), I2C::Error> {
        let mut maj_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_FIRM_MAJ], &mut maj_data)?;
        let mut min_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_FIRM_MIN], &mut min_data)?;
        Ok((maj_data[0], min_data[0]))
    }

    // slide 0x019B 411 knob 0x017B 379
    fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_WHOAMI], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    fn set_address(&mut self, new_address: u8) -> Result<(), SetAddressError<I2C::Error>> {
        if !(0x08..=0x77).contains(&new_address) {
            return Err(SetAddressError::ArgumentError);
        }
        self.i2c
            .write(self.address, &[REG_I2C_ADDRESS, new_address])
            .map_err(SetAddressError::I2cError)?;
        Ok(())
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod atmel_test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p22::{SetAddressError, P22};
    use crate::Atmel;

    #[test]
    pub fn set_led_on() {
        let expectations = [I2cTransaction::write(0x35, vec![0x87, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.set_led(true), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_led_off() {
        let expectations = [I2cTransaction::write(0x35, vec![0x87, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.set_led(false), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_off() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x07], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.get_led(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_on() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x07], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.get_led(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x01],
            vec![0x01, 0x10],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.whoami(), Ok(0x0110));
        i2c_clone.done();
    }

    #[test]
    pub fn firmware() {
        let expectations = [
            I2cTransaction::write_read(0x35, vec![0x02], vec![0x01]),
            I2cTransaction::write_read(0x35, vec![0x03], vec![0x02]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.firmware(), Ok((0x01, 0x02)));
        i2c_clone.done();
    }

    #[test]
    pub fn set_address() {
        let expectations = [I2cTransaction::write(0x09, vec![0x04, 0x69])];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p22 = P22 { i2c, address: 0x09 };
        p22.set_address(0x69).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_small() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p22 = P22 { i2c, address: 0x09 };
        assert_eq!(p22.set_address(0x07), Err(SetAddressError::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_large() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p22 = P22 { i2c, address: 0x09 };
        assert_eq!(p22.set_address(0x78), Err(SetAddressError::ArgumentError));

        i2c_clone.done();
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use crate::Driver;
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p22::P22;

    #[test]
    pub fn new() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P22::new(i2c, 0x35).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x05],
            vec![0xf0, 0x0d],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.read(), Ok(61453));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.self_test(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_not_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22 { i2c, address: 0x35 };

        assert_eq!(p22.self_test(), Ok(false));
        i2c_clone.done();
    }
}
