//! # Unofficial Rust Driver for PiicoDev Magnetometer
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Magnetometer-QMC6310
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-QMC6310-MicroPython-Module
//! [Official Product Site]: https://piico.dev/p15
//! [Datasheet]: https://datasheet.lcsc.com/lcsc/2007101835_QST-QMC6310U_C669299.pdf
use embedded_hal::i2c::I2c;
use rust_decimal::prelude::*;

const REG_XOUT: u8 = 0x01;
const REG_YOUT: u8 = 0x03;
const REG_ZOUT: u8 = 0x05;
const REG_STATUS: u8 = 0x09;
const REG_CONTROL1: u8 = 0x0A;
const REG_CONTROL2: u8 = 0x0B;
const REG_SIGN: u8 = 0x29;

const CONTROL1_VALUE: u8 = 0b1100_1101;
const CONTROL2_VALUE: u8 = 0b0000_0000;
const SIGN_VALUE: u8 = 0b0000_0110;

pub struct P15<I2C> {
    i2c: I2C,
    address: u8,
}

use crate::Driver;
impl<I2C: I2c> Driver<I2C, I2C::Error> for P15<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    fn init_inner(mut self) -> Result<Self, I2C::Error> {
        self.set_sign()?;
        self.set_range()?;
        self.set_control_register()?;
        Ok(self)
    }
}

impl<I2C: I2c> P15<I2C> {
    fn set_control_register(&mut self) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[REG_CONTROL1, CONTROL1_VALUE])?;

        Ok(())
    }

    fn set_range(&mut self) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[REG_CONTROL2, CONTROL2_VALUE])?;

        Ok(())
    }

    fn set_sign(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, &[REG_SIGN, SIGN_VALUE])?;

        Ok(())
    }

    pub fn data_ready(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_STATUS], &mut data)?;
        if (data[0] & 0b0000_0011) == 0x01 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn read(&mut self) -> Result<(Decimal, Decimal, Decimal), I2C::Error> {
        let mut data_x: [u8; 2] = [0, 0];
        self.i2c
            .write_read(self.address, &[REG_XOUT], &mut data_x)?;
        let mut data_y: [u8; 2] = [0, 0];
        self.i2c
            .write_read(self.address, &[REG_YOUT], &mut data_y)?;
        let mut data_z: [u8; 2] = [0, 0];
        self.i2c
            .write_read(self.address, &[REG_ZOUT], &mut data_z)?;
        Ok((
            Decimal::new(i16::from_le_bytes(data_x).into(), 1),
            Decimal::new(i16::from_le_bytes(data_y).into(), 1),
            Decimal::new(i16::from_le_bytes(data_z).into(), 1),
        ))
    }

    // should be moved to separate impl
    // pub fn read_angle(&mut self) -> Result<Angle, I2C::Error> {
    //     let (x, y, _) = self.read()?;
    //     Ok(Angle::from_radians(atan2(x, -y)))
    // }

    // pub fn read_magnitude(&mut self) -> Result<f64, I2C::Error> {
    //     let (x, y, z) = self.read()?;
    //     Ok(sqrt(x * x + y * y + z * z))
    // }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use rust_decimal::prelude::*;
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p15::P15;
    use crate::Driver;

    #[test]
    pub fn set_control_register() {
        let expectations = [I2cTransaction::write(0x1C, vec![0x0A, 0b1100_1101])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p15 = P15 { i2c, address: 0x1C };

        assert_eq!(p15.set_control_register(), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_sign() {
        let expectations = [I2cTransaction::write(0x1C, vec![0x29, 0x06])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p15 = P15 { i2c, address: 0x1C };

        assert_eq!(p15.set_sign(), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_range() {
        let expectations = [I2cTransaction::write(0x1C, vec![0x0B, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p15 = P15 { i2c, address: 0x1C };

        assert_eq!(p15.set_range(), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x1C, vec![0x29, 0x06]),
            I2cTransaction::write(0x1C, vec![0x0B, 0x00]),
            I2cTransaction::write(0x1C, vec![0x0A, 0b1100_1101]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P15::new(i2c, 0x1C).unwrap().init().unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn data_ready_true() {
        let expectations = [I2cTransaction::write_read(0x1C, vec![0x09], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p15 = P15 { i2c, address: 0x1C };
        assert_eq!(p15.data_ready(), Ok(true));

        i2c_clone.done();
    }

    #[test]
    pub fn data_ready_false() {
        let expectations = [I2cTransaction::write_read(0x1C, vec![0x09], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p15 = P15 { i2c, address: 0x1C };
        assert_eq!(p15.data_ready(), Ok(false));

        i2c_clone.done();
    }

    #[test]
    pub fn data_ready_overflow() {
        let expectations = [I2cTransaction::write_read(0x1C, vec![0x09], vec![0x03])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p15 = P15 { i2c, address: 0x1C };
        assert_eq!(p15.data_ready(), Ok(false));

        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [
            I2cTransaction::write_read(0x1C, vec![0x01], vec![0x10, 0x01]),
            I2cTransaction::write_read(0x1C, vec![0x03], vec![0xDF, 0xFD]),
            I2cTransaction::write_read(0x1C, vec![0x05], vec![0x30, 0x03]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p15 = P15 { i2c, address: 0x1C };
        assert_eq!(
            p15.read(),
            Ok((
                Decimal::new(272, 1),
                Decimal::new(-545, 1),
                Decimal::new(816, 1),
            ))
        );

        i2c_clone.done();
    }

    // #[test]
    // pub fn read_angle() {
    //     let expectations = [
    //         I2cTransaction::write_read(0x1C, vec![0x01], vec![0x10, 0x01]),
    //         I2cTransaction::write_read(0x1C, vec![0x03], vec![0xDF, 0xFD]),
    //         I2cTransaction::write_read(0x1C, vec![0x05], vec![0x30, 0x03]),
    //     ];
    //     let i2c = I2cMock::new(&expectations);
    //     let mut i2c_clone = i2c.clone();

    //     let mut p15 = P15 { i2c, address: 0x1C };
    //     assert_eq!(
    //         p15.read_angle(),
    //         Ok(Angle::from_degrees(26.522_983_798_797_82))
    //     );

    //     i2c_clone.done();
    // }

    // #[test]
    // pub fn read_magnitude() {
    //     let expectations = [
    //         I2cTransaction::write_read(0x1C, vec![0x01], vec![0x10, 0x01]),
    //         I2cTransaction::write_read(0x1C, vec![0x03], vec![0xDF, 0xFD]),
    //         I2cTransaction::write_read(0x1C, vec![0x05], vec![0x30, 0x03]),
    //     ];
    //     let i2c = I2cMock::new(&expectations);
    //     let mut i2c_clone = i2c.clone();

    //     let mut p15 = P15 { i2c, address: 0x1C };
    //     assert_eq!(p15.read_magnitude(), Ok(101.826_568_242_281_45_f64));

    //     i2c_clone.done();
    // }
}
