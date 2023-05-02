#![no_std]
#![warn(missing_docs)]

//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Precision-Temperature-Sensor-TMP117/tree/426af09299dc6ae9f254da7f45ef615f65c0f207
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-TMP117-MicroPython-Module/tree/2678a75ac4efbc8c9a16ceb55335108b04460996
//! [Official Product Site]: https://piico.dev/p1
//! [Datasheet]: https://www.ti.com/product/TMP117
//! # Unofficial Rust Driver for PiicoDev Precision Temperature Sensor
//! ## External Links
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]

use embedded_hal::i2c::I2c;
use measurements::Temperature;

/// Driver for PiicoDev P1
///
/// Typical usage:
///
/// 1. Create an instance through [`P1::new`]
/// 2. Read a temperature from the instance with [`P1::read`]
pub struct P1<I2C> {
    i2c: I2C,
    address: u8,
}

const REG_TEMPC: u8 = 0x0;

impl<I2C: I2c> P1<I2C> {
    /// Returns a new P1 driver instance
    ///
    /// - The [`I2c`] argument should be acquired from the target platform's HAL.
    /// - The address should match the hardware address set by the P1's onboard ASW switches.
    pub const fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Reads a [`Temperature`] from the P1
    /// # Errors
    ///
    /// Will return `I2C::Error` for any I2C communication errors.
    pub fn read(&mut self) -> Result<Temperature, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c.write_read(self.address, &[REG_TEMPC], &mut data)?;
        let temp_data_raw = u16::from_be_bytes(data);
        if temp_data_raw >= 0x8000 {
            Ok(Temperature::from_celsius(
                f64::from(temp_data_raw - 0x8000) * 7.8125e-3f64 - 256.0f64,
            ))
        } else {
            Ok(Temperature::from_celsius(
                f64::from(temp_data_raw) * 7.8125e-3f64,
            ))
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
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::P1;
    use measurements::Temperature;

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(0x48, vec![0], vec![0x0B, 0x86])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p1 = P1::new(i2c, 0x48);

        assert_eq!(p1.read(), Ok(Temperature::from_celsius(23.046_875)));
        i2c_clone.done();
    }

    #[test]
    pub fn read_negative() {
        let expectations = [I2cTransaction::write_read(0x48, vec![0], vec![0xF4, 0x7A])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p1 = P1::new(i2c, 0x48);

        assert_eq!(p1.read(), Ok(Temperature::from_celsius(-23.046_875)));
        i2c_clone.done();
    }
}
