#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![no_std]

use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::I2c;

pub struct P29<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
    address: u8,
}

impl<I2C: I2c, DELAY: DelayUs> P29<I2C, DELAY> {
    pub fn new(i2c: I2C, address: u8, delay: DELAY) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            delay,
            address,
        };
        res.reset()?;
        res.set_frequency(50)?;
        Ok(res)
    }

    pub fn reset(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, &[0x00, 0x00])
    }

    pub fn set_frequency(&mut self, frequency: u16) -> Result<(), I2C::Error> {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let prescale: u8 = (25_000_000.0 / 4096.0 / f64::from(frequency) + 0.5) as u8;
        let mut data: [u8; 1] = [0];
        self.i2c.write_read(self.address, &[0x00], &mut data)?;
        let old_mode: u8 = data[0];
        self.i2c
            .write(self.address, &[0x00, (old_mode & 0x7F) | 0x10])?;
        self.i2c.write(self.address, &[0xfe, prescale])?;
        self.i2c.write(self.address, &[0x00, old_mode])?;
        self.delay.delay_ms(1);

        self.i2c.write(self.address, &[0x00, old_mode | 0xA1])
    }

    pub fn get_pwm(&mut self, servo: u8) -> Result<(u16, u16), I2C::Error> {
        let mut data: [u8; 4] = [0; 4];
        self.i2c
            .write_read(self.address, &[0x06 + 4 * servo], &mut data)?;
        Ok((
            u16::from_le_bytes([data[0], data[1]]),
            u16::from_le_bytes([data[2], data[3]]),
        ))
    }

    pub fn set_pwm(&mut self, servo: u8, on: u16, off: u16) -> Result<(), I2C::Error> {
        let on_bytes: [u8; 2] = u16::to_le_bytes(on);
        let off_bytes: [u8; 2] = u16::to_le_bytes(off);
        self.i2c.write(
            self.address,
            &[
                0x06 + 4 * servo,
                on_bytes[0],
                on_bytes[1],
                off_bytes[0],
                off_bytes[1],
            ],
        )
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

    use crate::P29;

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x44, vec![0x00, 0x00]),
            I2cTransaction::write_read(0x44, vec![0x00], vec![0x70]),
            I2cTransaction::write(0x44, vec![0x00, 0x70]),
            I2cTransaction::write(0x44, vec![0xfe, 122]),
            I2cTransaction::write(0x44, vec![0x00, 0x70]),
            I2cTransaction::write(0x44, vec![0x00, 0xF1]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P29::new(i2c, 0x44, MockNoop {}).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn reset() {
        let expectations = [I2cTransaction::write(0x44, vec![0x00, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p29 = P29 {
            i2c,
            address: 0x44,
            delay: MockNoop {},
        };

        p29.reset().unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn set_pwm_0() {
        let expectations = [I2cTransaction::write(
            0x44,
            vec![0x06, 0x10, 0x02, 0x03, 0x01],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p29 = P29 {
            i2c,
            address: 0x44,
            delay: MockNoop {},
        };

        p29.set_pwm(0, 528, 259).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn set_pwm_1() {
        let expectations = [I2cTransaction::write(
            0x44,
            vec![0x0A, 0x10, 0x02, 0x03, 0x01],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p29 = P29 {
            i2c,
            address: 0x44,
            delay: MockNoop {},
        };

        p29.set_pwm(1, 528, 259).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn get_pwm_0() {
        let expectations = [I2cTransaction::write_read(
            0x44,
            vec![0x06],
            vec![0x23, 0x05, 0x34, 0x06],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p29 = P29 {
            i2c,
            address: 0x44,
            delay: MockNoop {},
        };

        assert_eq!(p29.get_pwm(0), Ok((1315, 1588)));
        i2c_clone.done();
    }

    #[test]
    pub fn get_pwm_1() {
        let expectations = [I2cTransaction::write_read(
            0x44,
            vec![0x0A],
            vec![0x23, 0x05, 0x34, 0x06],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p29 = P29 {
            i2c,
            address: 0x44,
            delay: MockNoop {},
        };

        assert_eq!(p29.get_pwm(1), Ok((1315, 1588)));
        i2c_clone.done();
    }

    #[test]
    pub fn set_frequency() {
        let expectations = [
            I2cTransaction::write_read(0x44, vec![0x00], vec![0x70]),
            I2cTransaction::write(0x44, vec![0x00, 0x70]),
            I2cTransaction::write(0x44, vec![0xfe, 122]),
            I2cTransaction::write(0x44, vec![0x00, 0x70]),
            I2cTransaction::write(0x44, vec![0x00, 0xF1]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p29 = P29 {
            i2c,
            address: 0x44,
            delay: MockNoop {},
        };

        p29.set_frequency(50).unwrap();
        i2c_clone.done();
    }
}
