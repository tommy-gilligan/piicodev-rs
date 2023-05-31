//! # Unofficial Rust Driver for PiicoDev Air Quality Sensor
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Air-Quality-Sensor-ENS160
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-ENS160-MicroPython-Module
//! [Official Product Site]: https://piico.dev/p23
//! [Datasheet]: https://github.com/CoreElectronics/CE-PiicoDev-Air-Quality-Sensor-ENS160/raw/main/Documents/ENS160-Datasheet%20v1.1.pdf
use embedded_hal::i2c::I2c;
use measurements::{Humidity, Temperature};

const REG_WHOAMI: u8 = 0x00;

const REG_OPMODE: u8 = 0x10;
const REG_CONFIG: u8 = 0x11;
const REG_TEMP_IN: u8 = 0x13;
const REG_RH_IN: u8 = 0x15;
const REG_DEVICE_STATUS: u8 = 0x20;

const DEVICE_ID: u16 = 0x0160;
const VAL_OPMODE_STANDARD: u8 = 0x02;

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

#[derive(PartialEq, Debug, Eq)]
pub struct AirQuality {
    pub aqi: u8,
    pub tvoc: i16,
    pub eco2: i16,
}

pub struct P23<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> P23<I2C> {
    pub fn new(i2c: I2C, address: u8) -> Result<Self, Error<I2C::Error>> {
        let mut res = Self { i2c, address };
        if res.whoami()? != DEVICE_ID {
            return Err(Error::UnexpectedDevice);
        }
        res.i2c
            .write(res.address, &[REG_OPMODE, VAL_OPMODE_STANDARD])?;
        res.i2c.write(res.address, &[REG_CONFIG, 0x00])?;
        res.set_temperature(Temperature::from_celsius(25.0))?;
        res.set_humidity(Humidity::from_percent(50.0))?;
        Ok(res)
    }

    pub fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_WHOAMI], &mut data)?;

        Ok(u16::from_le_bytes(data))
    }

    pub fn set_temperature(&mut self, temperature: Temperature) -> Result<(), I2C::Error> {
        let temperature_a: [u8; 2] = i16::to_le_bytes((temperature.as_kelvin() * 64.0) as i16);
        self.i2c.write(
            self.address,
            &[REG_TEMP_IN, temperature_a[0], temperature_a[1]],
        )?;
        Ok(())
    }

    pub fn set_humidity(&mut self, humidity: Humidity) -> Result<(), I2C::Error> {
        let humidity_a: [u8; 2] = i16::to_le_bytes((humidity.as_percent() * 512.0) as i16);
        self.i2c
            .write(self.address, &[REG_RH_IN, humidity_a[0], humidity_a[1]])?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<AirQuality, I2C::Error> {
        let mut data: [u8; 6] = [0; 6];
        self.i2c
            .write_read(self.address, &[REG_DEVICE_STATUS], &mut data)?;
        Ok(AirQuality {
            aqi: data[1],
            tvoc: i16::from_le_bytes([data[2], data[3]]),
            eco2: i16::from_le_bytes([data[4], data[5]]),
        })
    }

    pub fn data_ready(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 6] = [0; 6];
        self.i2c
            .write_read(self.address, &[REG_DEVICE_STATUS], &mut data)?;
        if (data[0] & 0b0000_0010) == 0x02 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p23::{AirQuality, Error, P23};
    use measurements::{Humidity, Temperature};

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write_read(0x53, vec![0x00], vec![0x60, 0x01]),
            I2cTransaction::write(0x53, vec![0x10, 0x02]),
            I2cTransaction::write(0x53, vec![0x11, 0x00]),
            I2cTransaction::write(0x53, vec![0x13, 0x89, 0x4A]),
            I2cTransaction::write(0x53, vec![0x15, 0x00, 0x64]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P23::new(i2c, 0x53).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn new_unexpected_device() {
        let expectations = [I2cTransaction::write_read(
            0x53,
            vec![0x00],
            vec![0x23, 0x86],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        assert_eq!(P23::new(i2c, 0x53).err(), Some(Error::UnexpectedDevice));

        i2c_clone.done();
    }

    #[test]
    pub fn set_temperature() {
        let expectations = [I2cTransaction::write(0x53, vec![0x13, 0x29, 0x48])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p23 = P23 { i2c, address: 0x53 };

        p23.set_temperature(Temperature::from_celsius(15.5))
            .unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_humidity() {
        let expectations = [I2cTransaction::write(0x53, vec![0x15, 0x00, 0x32])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p23 = P23 { i2c, address: 0x53 };

        p23.set_humidity(Humidity::from_percent(25.0)).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(
            0x53,
            vec![0x20],
            vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p23 = P23 { i2c, address: 0x53 };

        assert_eq!(
            p23.read(),
            Ok(AirQuality {
                aqi: 0x34,
                tvoc: 0x7856,
                eco2: -17254_i16
            })
        );

        i2c_clone.done();
    }

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(
            0x53,
            vec![0x00],
            vec![0x23, 0x86],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p23 = P23 { i2c, address: 0x53 };

        assert_eq!(p23.whoami(), Ok(0x8623));
        i2c_clone.done();
    }

    #[test]
    pub fn data_ready_true() {
        let expectations = [I2cTransaction::write_read(
            0x53,
            vec![0x20],
            vec![0x02, 0x00, 0x00, 0x00, 0x00, 0x00],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p23 = P23 { i2c, address: 0x53 };
        assert_eq!(p23.data_ready(), Ok(true));

        i2c_clone.done();
    }

    #[test]
    pub fn data_ready_false() {
        let expectations = [I2cTransaction::write_read(
            0x53,
            vec![0x20],
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p23 = P23 { i2c, address: 0x53 };
        assert_eq!(p23.data_ready(), Ok(false));

        i2c_clone.done();
    }
}
