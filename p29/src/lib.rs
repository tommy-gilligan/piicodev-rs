#![doc = include_str!("../README.md")]
#![no_std]
#![feature(lint_reasons)]
#![feature(int_roundings)]

use core::cmp;
use embedded_hal::{delay::DelayUs, i2c::I2c};
use measurements::Angle;

const FREQ: u32 = 50;
const PERIOD: u32 = 1_000_000 / FREQ;
const MIN_US: u32 = 600;
const MIN_DUTY: u32 = 4095 * MIN_US.div_ceil(PERIOD);
const MAX_US: u32 = 2400;
const MAX_DUTY: u32 = 4095 * MAX_US.div_ceil(PERIOD);
const DEGREES: f64 = 180.0;

#[must_use]
pub fn remap(old_val: i16, old_min: i16, old_max: i16, new_min: i16, new_max: i16) -> i16 {
    let x = (new_max - new_min) * (old_val - old_min) / (old_max - old_min) + new_min;
    cmp::min(new_max, cmp::max(x, new_min))
}
#[must_use]
pub fn us2duty(value: u16, period: u16) -> u16 {
    4095 * value / period
}

pub struct P29<I2C, DELAY> {
    i2c: I2C,
    address: u8,
    delay: DELAY,
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
        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
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

    pub fn set_pwm(&mut self, channel: u8, on: u16, off: u16) -> Result<(), I2C::Error> {
        let [msb_on, lsb_on]: [u8; 2] = u16::to_le_bytes(on);
        let [msb_off, lsb_off]: [u8; 2] = u16::to_le_bytes(off);
        self.i2c.write(
            self.address,
            &[0x06 + 4 * channel, msb_on, lsb_on, msb_off, lsb_off],
        )
    }

    pub fn set_speed(&mut self, channel: u8, _x: i16) -> Result<(), I2C::Error> {
        // let duty = remap(x, -1, 1, 4095 * 600 / (20_000), 4095 * 2400 / (20_000));
        // let [msb, lsb]: [u8; 2] = i16::to_be_bytes(duty);
        self.i2c
            .write(self.address, &[0x06 + 4 * channel, 0, 0, 0xeb, 0x01])
    }

    pub fn set_angle(&mut self, channel: u8, x: Angle) -> Result<(), I2C::Error> {
        let duty: f64 =
            f64::from(MIN_DUTY) + f64::from(MAX_DUTY - MIN_DUTY) * x.as_degrees() / DEGREES;
        let duty = cmp::min(MAX_DUTY, cmp::max(MIN_DUTY, duty as u32));
        self.set_duty(channel, duty as u16)?;
        Ok(())
    }

    pub fn set_duty(&mut self, channel: u8, value: u16) -> Result<(), I2C::Error> {
        self.set_pwm(channel, 0, value)?;
        Ok(())
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
    use measurements::Angle;

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
            I2cTransaction::write_read(0x44, vec![0x00], vec![0x00]),
            I2cTransaction::write(0x44, vec![0x00, 0x10]),
            I2cTransaction::write(0x44, vec![0xfe, 122]),
            I2cTransaction::write(0x44, vec![0x00, 0x00]),
            I2cTransaction::write(0x44, vec![0x00, 161]),
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

    #[test]
    pub fn set_speed() {
        let expectations = [I2cTransaction::write(
            0x44,
            vec![0x12, 0x00, 0x00, 0xeb, 0x01],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p29 = P29 {
            i2c,
            address: 0x44,
            delay: MockNoop {},
        };

        p29.set_speed(3, 1).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn set_angle() {
        let expectations = [I2cTransaction::write(
            0x44,
            vec![0x12, 0x00, 0x00, 0xff, 0x0f],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p29 = P29 {
            i2c,
            address: 0x44,
            delay: MockNoop {},
        };

        p29.set_angle(3, Angle::from_degrees(20.0)).unwrap();
        i2c_clone.done();
    }
}
