//! # Unofficial Rust Driver for PiicoDev 3-Axis Accelerometer
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! links missing from official docs
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Accelerometer-3-Axis-LIS3DH
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Accelerometer-LIS3DH-MicroPython-Module
//! [Official Product Site]: https://piico.dev/p26
//! [Datasheet]: https://core-electronics.com.au/attachments/uploads/lis3dh-datasheet.pdf
use embedded_hal::i2c::I2c;

const REG_WHOAMI: u8 = 0x0F;
const REG_CONTROL1: u8 = 0x20;
const REG_CONTROL3: u8 = 0x22;
const REG_CONTROL4: u8 = 0x23;
const REG_CONTROL5: u8 = 0x25;
const REG_STATUS: u8 = 0x27;
const OUT_X_L: u8 = 0x28;
const INT1_SRC: u8 = 0x31;
const CLICK_CFG: u8 = 0x38;
const CLICK_SRC: u8 = 0x39;
const CLICK_THS: u8 = 0x3A;
const DEVICE_ID: u8 = 0x33;
use num_enum::IntoPrimitive;

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

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum TapDetection {
    Disabled = 0x00,
    Single = 0x15,
    Double = 0x2A,
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Self::I2cError(error)
    }
}

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum Gravity {
    EarthTimes2 = 0b0000_0000,
}

use crate::Driver;
impl<I2C: I2c> Driver<I2C> for P26<I2C> {
    fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }
}

use fixed::types::I2F14;
impl<I2C: I2c> P26<I2C> {
    pub fn init(mut self) -> Result<Self, Error<I2C::Error>> {
        if self.whoami()? != DEVICE_ID {
            return Err(Error::UnexpectedDevice);
        }

        self.i2c.write(self.address, &[REG_CONTROL1, 0x07])?;
        self.i2c.write(self.address, &[REG_CONTROL4, 0x88])?;

        self.set_range(Gravity::EarthTimes2)?;
        self.set_rate(400)?;
        Ok(self)
    }

    pub fn data_ready(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[0x80 | REG_STATUS], &mut data)?;

        if (data[0] & 0b0000_1000) == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn whoami(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_WHOAMI], &mut data)?;

        Ok(data[0])
    }

    pub fn set_range(&mut self, range: Gravity) -> Result<(), I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[0x80 | REG_CONTROL4], &mut data)?;
        self.i2c.write(
            self.address,
            &[
                REG_CONTROL4,
                (data[0] & 0b1100_1111) | <Gravity as bitfield::Into<u8>>::into(range),
            ],
        )?;
        Ok(())
    }

    pub fn set_rate(&mut self, rate: u16) -> Result<(), Error<I2C::Error>> {
        let rr = match rate {
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
            .write_read(self.address, &[0x80 | REG_CONTROL1], &mut data)?;
        self.i2c
            .write(self.address, &[REG_CONTROL1, (data[0] & 0x0f) | rr])?;
        Ok(())
    }

    pub fn acceleration(&mut self) -> Result<(I2F14, I2F14, I2F14), I2C::Error> {
        let mut data: [u8; 6] = [0; 6];
        self.i2c
            .write_read(self.address, &[0x80 | OUT_X_L], &mut data)?;

        Ok((
            I2F14::from_bits(i16::from_le_bytes([data[0], data[1]])),
            I2F14::from_bits(i16::from_le_bytes([data[2], data[3]])),
            I2F14::from_bits(i16::from_le_bytes([data[4], data[5]])),
        ))
    }

    // pub fn angle(&mut self) -> Result<(Angle, Angle, Angle), I2C::Error> {
    //     let (x, y, z) = self.acceleration()?;
    //     Ok((
    //         Angle::from_radians(atan2(
    //             z.as_metres_per_second_per_second(),
    //             x.as_metres_per_second_per_second(),
    //         )),
    //         Angle::from_radians(atan2(
    //             x.as_metres_per_second_per_second(),
    //             y.as_metres_per_second_per_second(),
    //         )),
    //         Angle::from_radians(atan2(
    //             y.as_metres_per_second_per_second(),
    //             z.as_metres_per_second_per_second(),
    //         )),
    //     ))
    // }

    pub fn set_tap(
        &mut self,
        tap: TapDetection,
        threshold: u8,
        time_limit: u8,
        latency: u8,
        window: u8,
    ) -> Result<(), Error<I2C::Error>> {
        if threshold > 127 {
            return Err(Error::ArgumentError);
        }
        let mut data: [u8; 1] = [0; 1];
        match tap {
            TapDetection::Disabled => {
                self.i2c
                    .write_read(self.address, &[REG_CONTROL3 | 0x80], &mut data)?;
                self.i2c
                    .write(self.address, &[REG_CONTROL3, data[0] & 0x7F])?;
                Ok(self.i2c.write(self.address, &[CLICK_CFG, 0x00])?)
            }
            TapDetection::Single | TapDetection::Double => {
                self.i2c
                    .write_read(self.address, &[REG_CONTROL3 | 0x80], &mut data)?;
                self.i2c
                    .write(self.address, &[REG_CONTROL3, data[0] | 0x80])?;
                self.i2c.write(self.address, &[REG_CONTROL5, 0x08])?;
                self.i2c.write(self.address, &[CLICK_CFG, tap as u8])?;
                Ok(self.i2c.write(
                    self.address,
                    &[
                        CLICK_THS | 0x80,
                        threshold | 0x80,
                        time_limit,
                        latency,
                        window,
                    ],
                )?)
            }
        }
    }

    pub fn tapped(&mut self) -> Result<bool, Error<I2C::Error>> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[CLICK_SRC | 0x80], &mut data)?;
        if (data[0] & 0x40) == 0x00 {
            Ok(false)
        } else {
            self.i2c
                .write_read(self.address, &[INT1_SRC | 0x80], &mut data)?;
            Ok(true)
        }
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use crate::Driver;
    use fixed::types::I2F14;
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;

    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p26::{Error, Gravity, TapDetection, P26};

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

        P26::new(i2c, 0x19).init().unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn new_unexpected_device() {
        let expectations = [I2cTransaction::write_read(0x19, vec![0x0F], vec![0x32])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        assert_eq!(
            P26::new(i2c, 0x19).init().err(),
            Some(Error::UnexpectedDevice)
        );

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

        assert_eq!(p26.set_rate(400), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_rate_error() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.set_rate(401), Err(Error::ArgumentError));
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
                I2F14::lit("0.0469"),
                I2F14::lit("-0.0088"),
                I2F14::lit("1.0156")
            ))
        );
        i2c_clone.done();
    }

    // #[test]
    // pub fn angle() {
    //     let expectations = [I2cTransaction::write_read(
    //         0x19,
    //         vec![0xA8],
    //         vec![0x00, 0x03, 0x70, 0xff, 0x00, 0x41],
    //     )];
    //     let i2c = I2cMock::new(&expectations);
    //     let mut i2c_clone = i2c.clone();

    //     let mut p26 = P26 { i2c, address: 0x19 };

    //     assert_eq!(
    //         p26.angle(),
    //         Ok((
    //             Angle::from_radians(1.524_675_210_780_385_4_f64),
    //             Angle::from_radians(1.756_144_276_790_591_5_f64),
    //             Angle::from_radians(-0.008_653_630_137_437_27_f64),
    //         ))
    //     );
    //     i2c_clone.done();
    // }

    #[test]
    pub fn set_tap() {
        let expectations = [
            I2cTransaction::write_read(0x19, vec![0xA2], vec![0x9f]),
            I2cTransaction::write(0x19, vec![0x22, 0x1f]),
            I2cTransaction::write(0x19, vec![0x38, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.set_tap(TapDetection::Disabled, 40, 10, 80, 255), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_tap_threshold_too_large() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(
            p26.set_tap(TapDetection::Disabled, 128, 10, 80, 255),
            Err(Error::ArgumentError)
        );
        i2c_clone.done();
    }

    #[test]
    pub fn set_tap_single() {
        let expectations = [
            I2cTransaction::write_read(0x19, vec![0xA2], vec![0x0]),
            I2cTransaction::write(0x19, vec![0x22, 0x80]),
            I2cTransaction::write(0x19, vec![0x25, 0x08]),
            I2cTransaction::write(0x19, vec![0x38, 0x15]),
            I2cTransaction::write(0x19, vec![0xBA, 0xFF, 10, 80, 255]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.set_tap(TapDetection::Single, 127, 10, 80, 255), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_tap_double() {
        let expectations = [
            I2cTransaction::write_read(0x19, vec![0xA2], vec![0x0]),
            I2cTransaction::write(0x19, vec![0x22, 0x80]),
            I2cTransaction::write(0x19, vec![0x25, 0x08]),
            I2cTransaction::write(0x19, vec![0x38, 0x2A]),
            I2cTransaction::write(0x19, vec![0xBA, 0xFF, 15, 60, 200]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.set_tap(TapDetection::Double, 127, 15, 60, 200), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn tapped_false() {
        let expectations = [I2cTransaction::write_read(
            0x19,
            vec![0x39 | 0x80],
            vec![0x0],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.tapped(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn tapped_true() {
        let expectations = [
            I2cTransaction::write_read(0x19, vec![0x39 | 0x80], vec![0x44]),
            I2cTransaction::write_read(0x19, vec![0x31 | 0x80], vec![0xff]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p26 = P26 { i2c, address: 0x19 };

        assert_eq!(p26.tapped(), Ok(true));
        i2c_clone.done();
    }
}
