//! # Unofficial Rust Driver for PiicoDev Temperature Sensor
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Precision-Temperature-Sensor-TMP117/tree/426af09299dc6ae9f254da7f45ef615f65c0f207
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-TMP117-MicroPython-Module/tree/2678a75ac4efbc8c9a16ceb55335108b04460996
//! [Official Product Site]: https://piico.dev/p1
//! [Datasheet]: https://www.ti.com/product/TMP117

use crate::Driver;
use embedded_hal::i2c::I2c;
use fixed::types::I9F7;

const REG_TEMPC: u8 = 0x0;

pub struct P1<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> Driver<I2C, core::convert::Infallible> for P1<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<I2C: I2c> P1<I2C> {
    pub fn read(&mut self) -> Result<I9F7, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c.write_read(self.address, &[REG_TEMPC], &mut data)?;
        Ok(I9F7::from_bits(i16::from_be_bytes(data)))
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use fixed::types::I9F7;

    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p1::P1;
    use crate::Driver;

    #[test]
    pub fn new() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P1::new(i2c, 0x10).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(0x48, vec![0], vec![0x0B, 0x86])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p1 = P1 { i2c, address: 0x48 };

        assert_eq!(p1.read().unwrap(), I9F7::lit("23.05"));
        i2c_clone.done();
    }

    #[test]
    pub fn read_negative() {
        let expectations = [I2cTransaction::write_read(0x48, vec![0], vec![0xF4, 0x7A])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p1 = P1 { i2c, address: 0x48 };

        assert_eq!(p1.read().unwrap(), I9F7::lit("-23.05"));
        i2c_clone.done();
    }
}
