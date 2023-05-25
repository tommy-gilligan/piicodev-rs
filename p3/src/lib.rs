#![doc = include_str!("../README.md")]
#![no_std]
#![feature(lint_reasons)]

use embedded_hal::{delay::DelayUs, i2c::I2c};

const REG_ALS_CONF: u8 = 0x00;
const REG_ALS: u8 = 0x04;
const DEFAULT_SETTINGS: u8 = 0x00;

pub struct P3<I2C, DELAY> {
    i2c: I2C,
    address: u8,
    delay: DELAY,
}

impl<I2C: I2c, DELAY: DelayUs> P3<I2C, DELAY> {
    /// # Errors
    pub fn new(i2c: I2C, address: u8, delay: DELAY) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            address,
            delay,
        };
        res.i2c
            .write(res.address, &[REG_ALS_CONF, DEFAULT_SETTINGS])?;
        res.delay.delay_ms(4);
        Ok(res)
    }

    /// # Errors
    pub fn read(&mut self) -> Result<f64, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c.write_read(self.address, &[REG_ALS], &mut data)?;
        Ok(f64::from(u16::from_le_bytes(data)) + 0.0576_f64)
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

    use crate::P3;

    #[test]
    pub fn new() {
        let expectations = [I2cTransaction::write(0x10, vec![0, 0])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P3::new(i2c, 0x10, MockNoop {}).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [
            I2cTransaction::write(0x10, vec![0, 0]),
            I2cTransaction::write_read(0x10, vec![0x04], vec![0x02, 0x01]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p3 = P3::new(i2c, 0x10, MockNoop {}).unwrap();

        assert_eq!(p3.read().unwrap(), 258.0576_f64);

        i2c_clone.done();
    }
}
