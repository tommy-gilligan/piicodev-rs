#![no_std]

use embedded_hal::i2c::I2c;
use measurements::Distance;

#[derive(Copy, Clone)]
pub enum Address {
    X35 = 0x35,
}

pub struct P30<I2C> {
    i2c: I2C,
    address: Address,
    millimeters_per_microsecond: f64,
}

const REG_STATUS: u8 = 0x08;
const REG_FIRM_MAJ: u8 = 0x02;
const REG_FIRM_MIN: u8 = 0x03;
const REG_RAW: u8 = 0x05;
const REG_PERIOD: u8 = 0x06;
const REG_LED: u8 = 0x07;
const REG_SELF_TEST: u8 = 0x09;
const REG_WHOAMI: u8 = 0x01;

impl<I2C: I2c> P30<I2C> {
    pub fn new(i2c: I2C, address: Address) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            address,
            millimeters_per_microsecond: 0.343f64,
        };
        res.set_period(20)?;
        res.set_led(true)?;

        Ok(res)
    }

    pub fn distance(&mut self) -> Result<Distance, I2C::Error> {
        Ok(Distance::from_millimeters(
            f64::from(self.round_trip_time()?) * self.millimeters_per_microsecond / 2.0,
        ))
    }

    pub fn new_sample_available(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address as u8, &[REG_STATUS], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn round_trip_time(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address as u8, &[REG_RAW], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn get_period(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address as u8, &[REG_PERIOD], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn set_period(&mut self, period: u16) -> Result<(), I2C::Error> {
        let bytes: [u8; 2] = u16::to_be_bytes(period);
        self.i2c
            .write(self.address as u8, &[REG_PERIOD | 0x80, bytes[0], bytes[1]])?;
        Ok(())
    }

    pub fn get_led(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address as u8, &[REG_LED], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        if on {
            self.i2c.write(self.address as u8, &[REG_LED | 0x80, 1])?;
        } else {
            self.i2c.write(self.address as u8, &[REG_LED | 0x80, 0])?;
        }
        Ok(())
    }

    pub fn firmware(&mut self) -> Result<(u8, u8), I2C::Error> {
        let mut maj_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address as u8, &[REG_FIRM_MAJ], &mut maj_data)?;
        let mut min_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address as u8, &[REG_FIRM_MIN], &mut min_data)?;
        Ok((maj_data[0], min_data[0]))
    }

    pub fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address as u8, &[REG_WHOAMI], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn self_test(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address as u8, &[REG_SELF_TEST], &mut data)?;
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
    use measurements::Distance;

    use crate::{Address, P30};

    #[test]
    pub fn set_led_on() {
        let expectations = [I2cTransaction::write(0x35, vec![0x87, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.set_led(true), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_led_off() {
        let expectations = [I2cTransaction::write(0x35, vec![0x87, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.set_led(false), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_off() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x07], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.get_led(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_on() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x07], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.get_led(), Ok(true));
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

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.whoami(), Ok(0x0110));
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

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.firmware(), Ok((0x01, 0x02)));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.self_test(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn self_test_not_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.self_test(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn new_sample_available() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x08], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.new_sample_available(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn new_sample_unavailable() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x08], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.new_sample_available(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn round_trip_time() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x05],
            vec![0x9B, 0x2B],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.round_trip_time(), Ok(39723));
        i2c_clone.done();
    }

    #[test]
    pub fn set_period() {
        let expectations = [I2cTransaction::write(0x35, vec![0x86, 0x07, 0xD0])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.set_period(2000), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn get_period() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x06],
            vec![0x03, 0xE8],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.get_period(), Ok(1000));
        i2c_clone.done();
    }

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x35, vec![0x86, 0x00, 0x14]),
            I2cTransaction::write(0x35, vec![0x87, 0x01]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let p30 = P30::new(i2c, Address::X35).unwrap();

        assert_eq!(p30.millimeters_per_microsecond, 0.343f64);
        i2c_clone.done();
    }

    #[test]
    pub fn set_millimeters_per_microsecond() {
        let i2c = I2cMock::new(&[]);
        let mut i2c_clone = i2c.clone();
        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };
        p30.millimeters_per_microsecond = 0.890f64;
        assert_eq!(p30.millimeters_per_microsecond, 0.890f64);
        i2c_clone.done();
    }

    #[test]
    pub fn distance() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x05],
            vec![0x0B, 0x29],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: Address::X35,
            millimeters_per_microsecond: 3.2f64,
        };

        assert_eq!(p30.distance(), Ok(Distance::from_millimeters(4571.2f64)));
        i2c_clone.done();
    }
}
