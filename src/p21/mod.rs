//! # Unofficial Rust Driver for PiicoDev Button
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Button/tree/53c87f9c908d31c1385dfc4f9f4e1d9773aa05ae
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Switch-MicroPython-Module/tree/3bfbfa1ed58438afb9d7cb3032e24de1dc9742e7
//! [Official Product Site]: https://piico.dev/p21

use crate::Driver;
use embedded_hal::i2c::I2c;

const REG_IS_PRESSED: u8 = 0x11;
const REG_WAS_PRESSED: u8 = 0x12;
const REG_DOUBLE_PRESS_DETECTED: u8 = 0x13;
const REG_PRESS_COUNT: u8 = 0x14;
const REG_DOUBLE_PRESS_DURATION: u8 = 0x21;
const REG_EMA_SMOOTHING_FACTOR: u8 = 0x22;
const REG_EMA_PERIOD: u8 = 0x23;

pub struct P21<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> Driver<I2C, core::convert::Infallible> for P21<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<I2C: I2c> P21<I2C> {
    pub fn is_pressed(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_IS_PRESSED], &mut data)?;
        if data[0] == 1 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn was_double_pressed(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_DOUBLE_PRESS_DETECTED], &mut data)?;
        if data[0] == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn was_pressed(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_WAS_PRESSED], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn get_ema_smoothing_factor(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_EMA_SMOOTHING_FACTOR], &mut data)?;
        Ok(data[0])
    }

    pub fn set_ema_smoothing_factor(&mut self, smoothing_factor: u8) -> Result<(), I2C::Error> {
        self.i2c.write(
            self.address,
            &[REG_EMA_SMOOTHING_FACTOR | 0b1000_0000, smoothing_factor],
        )?;
        Ok(())
    }

    pub fn get_ema_period(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_EMA_PERIOD], &mut data)?;
        Ok(data[0])
    }

    pub fn set_ema_period(&mut self, period: u8) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[REG_EMA_PERIOD | 0b1000_0000, period])?;
        Ok(())
    }

    pub fn get_double_press_duration(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_DOUBLE_PRESS_DURATION], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn set_double_press_duration(
        &mut self,
        double_press_duration: u16,
    ) -> Result<(), I2C::Error> {
        let bytes: [u8; 2] = u16::to_be_bytes(double_press_duration);
        self.i2c.write(
            self.address,
            &[REG_DOUBLE_PRESS_DURATION | 0b1000_0000, bytes[0], bytes[1]],
        )?;
        Ok(())
    }

    pub fn press_count(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_PRESS_COUNT], &mut data)?;
        Ok(u16::from_be_bytes(data))
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

    use crate::p21::P21;

    #[test]
    pub fn new() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P21::new(i2c, 0x10).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn read_pressed() {
        let expectations = [I2cTransaction::write_read(0x10, vec![0x11], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.is_pressed(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn read_not_pressed() {
        let expectations = [I2cTransaction::write_read(0x10, vec![0x11], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.is_pressed(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn press_count() {
        let expectations = [I2cTransaction::write_read(
            0x10,
            vec![0x14],
            vec![0x01, 0x12],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.press_count(), Ok(274));
        i2c_clone.done();
    }

    #[test]
    pub fn get_double_press_duration() {
        let expectations = [I2cTransaction::write_read(
            0x10,
            vec![0x21],
            vec![0x02, 0x58],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.get_double_press_duration(), Ok(600));
        i2c_clone.done();
    }

    #[test]
    pub fn set_double_press_duration() {
        let expectations = [I2cTransaction::write(0x10, vec![0xA1, 0x00, 0x90])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        p21.set_double_press_duration(144).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn was_double_pressed_true() {
        let expectations = [I2cTransaction::write_read(0x10, vec![0x13], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.was_double_pressed(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn was_double_pressed_false() {
        let expectations = [I2cTransaction::write_read(0x10, vec![0x13], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.was_double_pressed(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn get_ema_smoothing_factor() {
        let expectations = [I2cTransaction::write_read(0x10, vec![0x22], vec![0x99])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.get_ema_smoothing_factor(), Ok(153));
        i2c_clone.done();
    }

    #[test]
    pub fn set_ema_smoothing_factor() {
        let expectations = [I2cTransaction::write(0x10, vec![0xA2, 0x66])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        p21.set_ema_smoothing_factor(102).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn get_ema_period() {
        let expectations = [I2cTransaction::write_read(0x10, vec![0x23], vec![0xAA])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.get_ema_period(), Ok(170));
        i2c_clone.done();
    }

    #[test]
    pub fn set_ema_period() {
        let expectations = [I2cTransaction::write(0x10, vec![0xA3, 0x77])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        p21.set_ema_period(119).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn was_pressed_true() {
        let expectations = [I2cTransaction::write_read(0x10, vec![0x12], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.was_pressed(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn was_pressed_false() {
        let expectations = [I2cTransaction::write_read(0x10, vec![0x12], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p21 = P21 { i2c, address: 0x10 };

        assert_eq!(p21.was_pressed(), Ok(false));
        i2c_clone.done();
    }
}

pub mod atmel;
pub mod whoami;
