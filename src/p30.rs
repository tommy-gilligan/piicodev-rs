//! # Unofficial Rust Driver for PiicoDev Ultrasonic Rangefinder
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Ultrasonic-Rangefinder/tree/3e006745fdc5123f8bc55bbcfde54461db72883c
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Ultrasonic-Rangefinder-MicroPython-Module/tree/6f5f4a65f41b29be7fd7c041cde15104fed2f31c
//! [Official Product Site]: https://piico.dev/p30
//! [Datasheet]: https://cdn.sparkfun.com/datasheets/Sensors/Proximity/HCSR04.pdf
use core::cell::Cell;
use embedded_hal::i2c::I2c;
use measurements::Length;

const REG_WHOAMI: u8 = 0x01;
const REG_FIRM_MAJ: u8 = 0x02;
const REG_FIRM_MIN: u8 = 0x03;
const REG_I2C_ADDRESS: u8 = 0x04;
const REG_RAW: u8 = 0x05;
const REG_PERIOD: u8 = 0x06;
const REG_LED: u8 = 0x07;
const REG_STATUS: u8 = 0x08;
const REG_SELF_TEST: u8 = 0x09;
const DEVICE_ID: u16 = 578;

pub struct P30<I2C> {
    i2c: I2C,
    address: u8,
    pub millimeters_per_microsecond: Cell<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    I2cError(E),
    ArgumentError,
    UnexpectedDevice,
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Self::I2cError(error)
    }
}

impl<I2C: I2c> P30<I2C> {
    pub fn new(i2c: I2C, address: u8) -> Result<Self, Error<I2C::Error>> {
        let mut res = Self {
            i2c,
            address,
            millimeters_per_microsecond: Cell::new(0.343_f64),
        };
        if res.whoami()? != DEVICE_ID {
            return Err(Error::UnexpectedDevice);
        }
        res.set_period(20)?;
        res.set_led(true)?;

        Ok(res)
    }

    /// Returns the [`Length`] between the ultrasound sensor and the surface of a target.
    pub fn length(&mut self) -> Result<Length, I2C::Error> {
        Ok(Length::from_millimeters(
            f64::from(self.round_trip_time()?) * self.millimeters_per_microsecond.get() / 2.0,
        ))
    }

    pub fn new_sample_available(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_STATUS], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn round_trip_time(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c.write_read(self.address, &[REG_RAW], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn get_period(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_PERIOD], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn set_period(&mut self, period: u16) -> Result<(), I2C::Error> {
        let bytes: [u8; 2] = u16::to_be_bytes(period);
        self.i2c
            .write(self.address, &[REG_PERIOD | 0x80, bytes[0], bytes[1]])?;
        Ok(())
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

    // 0x0242 578
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

    // should somehow destroy instance after call
    pub fn set_address(&mut self, new_address: u8) -> Result<(), Error<I2C::Error>> {
        if !(0x08..=0x77).contains(&new_address) {
            return Err(Error::ArgumentError);
        }
        self.i2c
            .write(self.address, &[REG_I2C_ADDRESS, new_address])?;
        Ok(())
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use core::cell::Cell;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use measurements::Length;

    use crate::p30::{Error, P30};

    #[test]
    pub fn self_test_ok() {
        let expectations = [I2cTransaction::write_read(0x35, vec![0x09], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
        };

        assert_eq!(p30.get_period(), Ok(1000));
        i2c_clone.done();
    }

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write_read(0x35, vec![0x01], vec![0x02, 0x42]),
            I2cTransaction::write(0x35, vec![0x86, 0x00, 0x14]),
            I2cTransaction::write(0x35, vec![0x87, 0x01]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let p30 = P30::new(i2c, 0x35).unwrap();

        assert_eq!(p30.millimeters_per_microsecond.get(), 0.343_f64);
        i2c_clone.done();
    }

    #[test]
    pub fn new_unexpected_device() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x01],
            vec![0x01, 0x10],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        assert_eq!(P30::new(i2c, 0x35).err(), Some(Error::UnexpectedDevice));
        i2c_clone.done();
    }

    #[test]
    pub fn set_millimeters_per_microsecond() {
        let i2c = I2cMock::new(&[]);
        let mut i2c_clone = i2c.clone();
        let p30 = P30 {
            i2c,
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
        };
        p30.millimeters_per_microsecond.set(0.890_f64);
        assert_eq!(p30.millimeters_per_microsecond.get(), 0.890_f64);
        i2c_clone.done();
    }

    #[test]
    pub fn length() {
        let expectations = [I2cTransaction::write_read(
            0x35,
            vec![0x05],
            vec![0x0B, 0x29],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
        };

        assert_eq!(p30.length(), Ok(Length::from_millimeters(4571.2_f64)));
        i2c_clone.done();
    }

    #[test]
    pub fn set_address() {
        let expectations = [I2cTransaction::write(0x35, vec![0x04, 0x69])];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p30 = P30 {
            i2c,
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
        };
        p30.set_address(0x69).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_small() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p30 = P30 {
            i2c,
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
        };
        assert_eq!(p30.set_address(0x07), Err(Error::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_large() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p30 = P30 {
            i2c,
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
        };
        assert_eq!(p30.set_address(0x78), Err(Error::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn set_led_on() {
        let expectations = [I2cTransaction::write(0x35, vec![0x87, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p30 = P30 {
            i2c,
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
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
            address: 0x35,
            millimeters_per_microsecond: Cell::new(3.2_f64),
        };

        assert_eq!(p30.firmware(), Ok((0x01, 0x02)));
        i2c_clone.done();
    }
}
