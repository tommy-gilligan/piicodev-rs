//! # Unofficial Rust Driver for `PiicoDev` Ambient Light Sensor
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//! - [Alternate Driver]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Ambient-Light-Sensor-VEML6030/tree/2c46d51e90e8e83d5c3dfa3b6a614adb75469b6c
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-VEML6030-MicroPython-Module/tree/14b19d9dffe959efd90a55e7a37e663788ab53ff
//! [Official Product Site]: https://piico.dev/p3
//! [Datasheet]: https://www.vishay.com/en/product/84366/
//! [Alternate Driver]: https://github.com/eldruin/veml6030-rs

use crate::Driver;
use embedded_hal::i2c::I2c;

const REG_ALS_CONF: u8 = 0x00;
const REG_ALS: u8 = 0x04;
const DEFAULT_SETTINGS: u8 = 0x00;

pub struct P3<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> Driver<I2C, I2C::Error> for P3<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    fn init_inner(mut self) -> Result<Self, I2C::Error> {
        self.i2c
            .write(self.address, &[REG_ALS_CONF, DEFAULT_SETTINGS])?;
        Ok(self)
    }
}

impl<I2C: I2c> P3<I2C> {
    pub fn read(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c.write_read(self.address, &[REG_ALS], &mut data)?;
        Ok(u16::from_le_bytes(data))
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

    use crate::p3::P3;

    #[test]
    pub fn new() {
        let expectations = [I2cTransaction::write(0x10, vec![0, 0])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P3::new(i2c, 0x10).unwrap().init().unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(
            0x10,
            vec![0x04],
            vec![0x02, 0x01],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p3 = P3 { i2c, address: 0x10 };

        assert_eq!(p3.read().unwrap(), 258);

        i2c_clone.done();
    }
}
