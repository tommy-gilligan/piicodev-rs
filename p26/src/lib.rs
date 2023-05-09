#![doc = include_str!("../README.md")]
#![no_std]
#![feature(lint_reasons)]

use embedded_hal::i2c::I2c;
use measurements::{Acceleration, Angle, Frequency};

const R_WHOAMI: u8 = 0x0F;
const DEVICE_ID: u8 = 0x33;
const I2C_ADDRESS: u8 = 0x19;
const OUT_X_L: u8 = 0x28;
const CTRL_REG_1: u8 = 0x20;
const CTRL_REG_2: u8 = 0x21;
const CTRL_REG_3: u8 = 0x22;
const CTRL_REG_4: u8 = 0x23;
const CTRL_REG_5: u8 = 0x25;
const INT1_SRC: u8 = 0x31;
const CLICK_CFG: u8 = 0x38;
const CLICKSRC: u8 = 0x39;
const CLICK_THS: u8 = 0x3A;
const TIME_LIMIT: u8 = 0x3B;
const TIME_LATENCY: u8 = 0x3C;
const TIME_WINDOW: u8 = 0x3D;
const STATUS_REG: u8 = 0x27;

pub struct P26<I2C> {
    i2c: I2C,
    address: u8,
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

use libm::atan2;

pub enum Gravity {
    EarthTimes2 = 0b0000_0000,
}

impl<I2C: I2c> P26<I2C> {
    pub fn new(i2c: I2C, address: u8) -> Result<Self, Error<I2C::Error>> {
        let mut res = Self { i2c, address };
        if res.whoami()? != DEVICE_ID {
            return Err(Error::UnexpectedDevice);
        }

        res.i2c.write(address, &[CTRL_REG_1, 0x07])?;
        res.i2c.write(address, &[CTRL_REG_4, 0x88])?;

        res.set_range(Gravity::EarthTimes2)?;
        res.set_rate(Frequency::from_hertz(400.0))?;
        Ok(res)
    }

    pub fn data_ready(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[0x80 | STATUS_REG], &mut data)?;

        if (data[0] & 0b0000_1000) != 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn whoami(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c.write_read(self.address, &[R_WHOAMI], &mut data)?;

        Ok(data[0])
    }

    pub fn set_range(&mut self, range: Gravity) -> Result<(), I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[0x80 | CTRL_REG_4], &mut data)?;
        self.i2c.write(
            self.address,
            &[CTRL_REG_4, (data[0] & 0b1100_1111) | (range as u8)],
        )?;
        Ok(())
    }

    pub fn set_rate(&mut self, rate: Frequency) -> Result<(), Error<I2C::Error>> {
        let rr = match rate.as_hertz() as u16 {
            0 => 0x00,
            1 => 0x10,
            10 => 0x20,
            25 => 0x30,
            50 => 0x40,
            100 => 0x50,
            200 => 0x60,
            400 => 0x70,
            _ => return Err(Error::ArgumentError),
        };
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[0x80 | CTRL_REG_1], &mut data)?;
        self.i2c
            .write(self.address, &[CTRL_REG_1, (data[0] & 0x0f) | rr])?;
        Ok(())
    }

    pub fn acceleration(
        &mut self,
    ) -> Result<(Acceleration, Acceleration, Acceleration), I2C::Error> {
        let mut data: [u8; 6] = [0; 6];
        let den: f64 = 1670.295;
        self.i2c
            .write_read(self.address, &[0x80 | OUT_X_L], &mut data)?;

        Ok((
            Acceleration::from_metres_per_second_per_second(
                f64::from(i16::from_le_bytes([data[0], data[1]])) / den,
            ),
            Acceleration::from_metres_per_second_per_second(
                f64::from(i16::from_le_bytes([data[2], data[3]])) / den,
            ),
            Acceleration::from_metres_per_second_per_second(
                f64::from(i16::from_le_bytes([data[4], data[5]])) / den,
            ),
        ))
    }

    pub fn angle(&mut self) -> Result<(Angle, Angle, Angle), I2C::Error> {
        let (x, y, z) = self.acceleration()?;
        Ok((
            Angle::from_radians(atan2(
                z.as_metres_per_second_per_second(),
                x.as_metres_per_second_per_second(),
            )),
            Angle::from_radians(atan2(
                x.as_metres_per_second_per_second(),
                y.as_metres_per_second_per_second(),
            )),
            Angle::from_radians(atan2(
                y.as_metres_per_second_per_second(),
                z.as_metres_per_second_per_second(),
            )),
        ))
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
    use measurements::{Acceleration, Angle, Frequency};

    use crate::{Error, Gravity, P26};

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write_read(0x19, vec![0x0F], vec![0x33]),
            I2cTransaction::write(0x19, vec![0x20, 0x07]),
            I2cTransaction::write(0x19, vec![0x23, 0x88]),
            I2cTransaction::write_read(0x19, vec![0xA3], vec![136]),
            I2cTransaction::write(0x19, vec![0x23, 0x88]),
            I2cTransaction::write_read(0x19, vec![0xA0], vec![0x07]),
            I2cTransaction::write(0x19, vec![0x20, 0x77]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let _p26 = P26::new(i2c, 0x19).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn new_unexpected_device() {
        let expectations = [I2cTransaction::write_read(0x19, vec![0x0F], vec![0x32])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        assert_eq!(P26::new(i2c, 0x19).err(), Some(Error::UnexpectedDevice));

        i2c_clone.done();
    }

    #[test]
    pub fn set_range() {
        let expectations = [
            I2cTransaction::write_read(0x19, vec![0xA3], vec![136]),
            I2cTransaction::write(0x19, vec![0x23, 0x88]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.set_range(Gravity::EarthTimes2), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_rate() {
        let expectations = [
            I2cTransaction::write_read(0x19, vec![0xA0], vec![0x07]),
            I2cTransaction::write(0x19, vec![0x20, 0x77]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.set_rate(Frequency::from_hertz(400.0)), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_rate_error() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(
            p26.set_rate(Frequency::from_hertz(401.1)),
            Err(Error::ArgumentError)
        );
        i2c_clone.done();
    }

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(0x19, vec![0x0F], vec![0x33])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.whoami(), Ok(0x33));
        i2c_clone.done();
    }

    #[test]
    pub fn data_ready_true() {
        let expectations = [I2cTransaction::write_read(0x19, vec![0xA7], vec![0x08])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.data_ready(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn data_ready_false() {
        let expectations = [I2cTransaction::write_read(0x19, vec![0xA7], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.data_ready(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn acceleration() {
        let expectations = [I2cTransaction::write_read(
            0x19,
            vec![0xA8],
            vec![0x00, 0x03, 0x70, 0xff, 0x00, 0x41],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(
            p26.acceleration(),
            Ok((
                Acceleration::from_meters_per_second_per_second(0.459_799_017_538_817_97_f64),
                Acceleration::from_meters_per_second_per_second(-0.086_212_315_788_528_37_f64),
                Acceleration::from_meters_per_second_per_second(9.962_312_046_674_39_f64),
            ))
        );
        i2c_clone.done();
    }

    #[test]
    pub fn angle() {
        let expectations = [I2cTransaction::write_read(
            0x19,
            vec![0xA8],
            vec![0x00, 0x03, 0x70, 0xff, 0x00, 0x41],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(
            p26.angle(),
            Ok((
                Angle::from_radians(1.524_675_210_780_385_4_f64),
                Angle::from_radians(1.756_144_276_790_591_5_f64),
                Angle::from_radians(-0.008_653_630_137_437_27_f64),
            ))
        );
        i2c_clone.done();
    }
}
