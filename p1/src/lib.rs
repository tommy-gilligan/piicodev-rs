#![no_std]
#![warn(missing_docs)]

//! Unofficial Rust Driver for [PiicoDev P1](https://piico.dev/p1) (TMP117)

use embedded_hal::i2c::I2c;
use measurements::Temperature;

/// P1 Hardware Addresses
///
/// The hardware address of the P1 should be set with the onboard ASW switches.  Please refer to
/// the [schematic](https://piico.dev/p1).
#[derive(Copy, Clone)]
pub enum Address {
    /// Hardware address `0x48` is active when the ASW is set as `1: On, 2: Off, 3: Off, 4: Off`
    X48 = 0x48,
    /// Hardware address `0x49` is active when the ASW is set as `1: Off, 2: On, 3: Off, 4: Off`
    X49 = 0x49,
    /// Hardware address `0x4A` is active when the ASW is set as `1: Off, 2: Off, 3: On, 4: Off`
    X4A = 0x4A,
    /// Hardware address `0x4B` is active when the ASW is set as `1: Off, 2: Off, 3: Off, 4: On`
    X4B = 0x4B,
}

/// The P1 driver
///
/// Typical usage:
///
/// 1. Create an instance through [`P1::new`]
/// 2. Read a temperature from the instance with [`P1::read`]
///
pub struct P1<I2C> {
    i2c: I2C,
    address: Address,
}

const REG_TEMPC: u8 = 0x0;

impl<I2C: I2c> P1<I2C> {
    /// Returns a new P1 driver instance
    ///
    /// The [`I2c`] argument should be acquired from the target platform's HAL.
    /// The address should match the hardware address set by the P1's onboard ASW switches.
    pub const fn new(i2c: I2C, address: Address) -> Self {
        Self { i2c, address }
    }

    /// Reads a [`Temperature`] from the P1
    ///
    /// # Errors
    ///
    /// the errors
    pub fn read(&mut self) -> Result<Temperature, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c
            .write_read(self.address as u8, &[REG_TEMPC], &mut data)?;
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

    use crate::{Address, P1};
    use measurements::Temperature;

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(0x48, vec![0], vec![0x0B, 0x86])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p1 = P1::new(i2c, Address::X48);

        assert_eq!(p1.read(), Ok(Temperature::from_celsius(23.046_875)));
        i2c_clone.done();
    }

    #[test]
    pub fn read_negative() {
        let expectations = [I2cTransaction::write_read(0x48, vec![0], vec![0xF4, 0x7A])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p1 = P1::new(i2c, Address::X48);

        assert_eq!(p1.read(), Ok(Temperature::from_celsius(-23.046_875)));
        i2c_clone.done();
    }
}
