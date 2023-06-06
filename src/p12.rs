//! # Unofficial Rust Driver for PiicoDev Capacitive Touch Sensor
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Capacitive-Touch-Sensor-CAP1203/tree/1178346d1c3d1f11ed98f5183aa7f7c944a775a6
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-CAP1203-MicroPython-Module/tree/f2a061b83e020ef96865ba97215793c02717747e
//! [Official Product Site]: https://piico.dev/p12
//! [Datasheet]: https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ProductDocuments/DataSheets/00001572B.pdf
use embedded_hal::i2c::I2c;

const MAIN_CONTROL: u8 = 0x00;
const GENERAL_STATUS: u8 = 0x02;
const SENSOR_INPUT_STATUS: u8 = 0x03;
const SENSOR_INPUT_1_DELTA_COUNT: u8 = 0x10;
const SENSOR_INPUT_2_DELTA_COUNT: u8 = 0x11;
const SENSOR_INPUT_3_DELTA_COUNT: u8 = 0x12;
const SENSITIVITY_CONTROL: u8 = 0x1F;
const MULTIPLE_TOUCH_CONFIG: u8 = 0x2A;

pub enum TouchMode {
    Single = 0xFF,
    Multi = 0x7F,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    InvalidSensitivity,
    I2cError(E),
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Self::I2cError(error)
    }
}

pub struct P12<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> P12<I2C> {
    /// # Errors
    pub fn new(
        i2c: I2C,
        address: u8,
        touch_mode: Option<TouchMode>,
        sensitivity: Option<u8>,
    ) -> Result<Self, Error<I2C::Error>> {
        let mut res = Self { i2c, address };

        let mut data: [u8; 1] = [0];
        res.i2c
            .write_read(res.address, &[MULTIPLE_TOUCH_CONFIG], &mut data)?;
        // not working
        res.i2c.write(
            res.address,
            &[
                MULTIPLE_TOUCH_CONFIG,
                (touch_mode.unwrap_or(TouchMode::Multi) as u8) & data[0],
            ],
        )?;
        res.set_sensitivity(sensitivity.unwrap_or(3))?;

        Ok(res)
    }

    /// # Errors
    pub fn get_sensitivity(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[SENSITIVITY_CONTROL], &mut data)?;
        Ok(data[0])
    }

    /// # Errors
    pub fn set_sensitivity(&mut self, sensitivity: u8) -> Result<(), Error<I2C::Error>> {
        if sensitivity > 7 {
            return Err(Error::InvalidSensitivity);
        }
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[SENSITIVITY_CONTROL], &mut data)?;
        self.i2c.write(
            self.address,
            &[
                SENSITIVITY_CONTROL,
                (data[0] & 0x8F) | (sensitivity << 4_u8),
            ],
        )?;
        Ok(())
    }

    /// # Errors
    pub fn clear_interrupt(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c.write(self.address, &[MAIN_CONTROL, 0x00])?;
        self.i2c
            .write_read(self.address, &[MAIN_CONTROL], &mut data)?;
        Ok(data[0])
    }

    /// # Errors
    pub fn read(&mut self) -> Result<(bool, bool, bool), I2C::Error> {
        self.clear_interrupt()?;
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[GENERAL_STATUS], &mut data)?;
        self.i2c
            .write_read(self.address, &[SENSOR_INPUT_STATUS], &mut data)?;
        Ok((
            (data[0] & 0b0000_0001) != 0x00,
            (data[0] & 0b0000_0010) != 0x00,
            (data[0] & 0b0000_0100) != 0x00,
        ))
    }

    /// # Errors
    pub fn read_delta_counts(&mut self) -> Result<(i8, i8, i8), I2C::Error> {
        let mut data_0: [u8; 1] = [0];
        let mut data_1: [u8; 1] = [0];
        let mut data_2: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[SENSOR_INPUT_1_DELTA_COUNT], &mut data_0)?;
        self.i2c
            .write_read(self.address, &[SENSOR_INPUT_2_DELTA_COUNT], &mut data_1)?;
        self.i2c
            .write_read(self.address, &[SENSOR_INPUT_3_DELTA_COUNT], &mut data_2)?;

        #[expect(clippy::cast_possible_wrap)]
        return Ok((data_0[0] as i8, data_1[0] as i8, data_2[0] as i8));
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal::i2c::ErrorKind;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p12::{Error, P12};

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write_read(0x28, vec![0x2A], vec![0xF4]),
            I2cTransaction::write(0x28, vec![0x2A, 0x74]),
            I2cTransaction::write_read(0x28, vec![0x1F], vec![0x70]),
            I2cTransaction::write(0x28, vec![0x1F, 0x30]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P12::new(i2c, 0x28, None, None).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn get_sensitivity() {
        let expectations = [I2cTransaction::write_read(0x28, vec![0x1F], vec![0x87])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p12 = P12 { i2c, address: 0x28 };
        assert_eq!(p12.get_sensitivity(), Ok(0x87));

        i2c_clone.done();
    }

    #[test]
    pub fn set_sensitivity() {
        let expectations = [
            I2cTransaction::write_read(0x28, vec![0x1F], vec![0x87]),
            I2cTransaction::write(0x28, vec![0x1F, 0xC7]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p12 = P12 { i2c, address: 0x28 };
        p12.set_sensitivity(4).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_sensitivity_with_error() {
        let i2c_error = ErrorKind::Other;
        let expectations =
            [I2cTransaction::write_read(0x28, vec![0x1F], vec![0x87]).with_error(i2c_error)];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p12 = P12 { i2c, address: 0x28 };
        assert_eq!(p12.set_sensitivity(4), Err(Error::I2cError(i2c_error)));

        i2c_clone.done();
    }

    #[test]
    pub fn set_sensitivity_error() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p12 = P12 { i2c, address: 0x28 };
        assert_eq!(p12.set_sensitivity(10), Err(Error::InvalidSensitivity));

        i2c_clone.done();
    }

    #[test]
    pub fn clear_interrupt() {
        let expectations = [
            I2cTransaction::write(0x28, vec![0x00, 0x00]),
            I2cTransaction::write_read(0x28, vec![0x00], vec![0xF4]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p12 = P12 { i2c, address: 0x28 };
        assert_eq!(p12.clear_interrupt(), Ok(0xF4));

        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [
            I2cTransaction::write(0x28, vec![0x00, 0x00]),
            I2cTransaction::write_read(0x28, vec![0x00], vec![0xF4]),
            I2cTransaction::write_read(0x28, vec![0x02], vec![0xC4]),
            I2cTransaction::write_read(0x28, vec![0x03], vec![0b0000_0101]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p12 = P12 { i2c, address: 0x28 };
        assert_eq!(p12.read(), Ok((true, false, true)));

        i2c_clone.done();
    }

    #[test]
    pub fn read_delta_counts() {
        let expectations = [
            I2cTransaction::write_read(0x28, vec![0x10], vec![0xFB]),
            I2cTransaction::write_read(0x28, vec![0x11], vec![0xC8]),
            I2cTransaction::write_read(0x28, vec![0x12], vec![93]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p12 = P12 { i2c, address: 0x28 };
        assert_eq!(p12.read_delta_counts(), Ok((-5, -56, 93)));

        i2c_clone.done();
    }
}