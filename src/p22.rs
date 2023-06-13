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

use crate::Driver;
use embedded_hal::i2c::I2c;

const REG_POT: u8 = 0x05;
const REG_SELF_TEST: u8 = 0x09;

pub struct P22<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> Driver<I2C, core::convert::Infallible> for P22<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
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

pub mod atmel;
pub mod whoami;
