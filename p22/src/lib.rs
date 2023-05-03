#![doc = include_str!("../README.md")]
#![no_std]
#![feature(lint_reasons)]

use embedded_hal::i2c::I2c;

pub struct P22<I2C> {
    i2c: I2C,
    address: u8,
}

const REG_WHOAMI: u8 = 0x01;
const REG_FIRM_MAJ: u8 = 0x02;
const REG_FIRM_MIN: u8 = 0x03;
const REG_POT: u8 = 0x05;
const REG_LED: u8 = 0x07;
const REG_SELF_TEST: u8 = 0x09;

impl<I2C: I2c> P22<I2C> {
    pub const fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    pub fn read(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c.write_read(self.address, &[REG_POT], &mut data)?;
        Ok(u16::from_be_bytes(data))
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

    pub fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_WHOAMI], &mut data)?;
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
#[macro_use]
extern crate std;

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::P22;

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x05],
            vec![0xf0, 0x0d],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22::new(i2c, 0x35);

        assert_eq!(p22.read(), Ok(61453));
        i2c_clone.done();
    }

    #[test]
    pub fn set_led_on() {
        let expectations = [I2cTransaction::write(0x35, vec![0x87, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22::new(i2c, 0x35);

        assert_eq!(p22.set_led(true), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_led_off() {
        let expectations = [I2cTransaction::write(0x35, vec![0x87, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22::new(i2c, 0x35);

        assert_eq!(p22.set_led(false), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_off() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x07], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22::new(i2c, 0x35);

        assert_eq!(p22.get_led(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_on() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x07], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22::new(i2c, 0x35);

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

        let mut p22 = P22::new(i2c, 0x35);

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

        let mut p22 = P22::new(i2c, 0x35);

        assert_eq!(p22.firmware(), Ok((0x01, 0x02)));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22::new(i2c, 0x35);

        assert_eq!(p22.self_test(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_not_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p22 = P22::new(i2c, 0x35);

        assert_eq!(p22.self_test(), Ok(false));
        i2c_clone.done();
    }
}
