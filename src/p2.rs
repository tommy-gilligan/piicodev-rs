//! # Unofficial Rust Driver for PiicoDev Atmospheric Sensor
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Atmospheric-Sensor-BME280
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-BME280-MicroPython-Module
//! [Official Product Site]: https://piico.dev/p2
//! [Datasheet]: https://core-electronics.com.au/attachments/uploads/bme280.pdf
use cast::u32;
use core::num::NonZeroI64;
use embedded_hal::i2c::I2c;

const REG_TEMP: u8 = 0xFA;
const REG_PRESS: u8 = 0xF7;
const REG_HUM: u8 = 0xFD;

const DIG_T: u8 = 0x88;
const DIG_P: u8 = 0x8E;
const DIG_H: u8 = 0xE1;

pub type DigT = (u16, i16, i16);
pub type DigP = (u16, i16, i16, i16, i16, i16, i16, i16, i16);
pub type DigH = (u8, i16, u8, i16, i16, i8);

pub struct P2<I2C> {
    i2c: I2C,
    address: u8,
    pub temperature_data: Option<DigT>,
    pub pressure_data: Option<DigP>,
    pub humidity_data: Option<DigH>,
    pub t_fine: Option<i32>,
}

use crate::Driver;
impl<I2C: I2c> Driver<I2C> for P2<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            temperature_data: None,
            pressure_data: None,
            humidity_data: None,
            t_fine: None,
        }
    }
}

impl<I2C: I2c> P2<I2C> {
    pub fn init(mut self) -> Result<Self, I2C::Error> {
        self.load_temperature_data()?;
        self.load_pressure_data()?;
        self.load_humidity_data()?;
        Ok(self)
    }

    fn load_temperature_data(&mut self) -> Result<(), I2C::Error> {
        let mut dig_t: [u8; 6] = [0; 6];
        self.i2c.write_read(self.address, &[DIG_T], &mut dig_t)?;
        self.temperature_data = Some((
            u16::from_le_bytes([dig_t[0], dig_t[1]]),
            i16::from_le_bytes([dig_t[2], dig_t[3]]),
            i16::from_le_bytes([dig_t[4], dig_t[5]]),
        ));
        Ok(())
    }

    fn load_pressure_data(&mut self) -> Result<(), I2C::Error> {
        let mut dig_p: [u8; 18] = [0; 18];
        self.i2c.write_read(self.address, &[DIG_P], &mut dig_p)?;
        self.pressure_data = Some((
            u16::from_le_bytes([dig_p[0], dig_p[1]]),
            i16::from_le_bytes([dig_p[2], dig_p[3]]),
            i16::from_le_bytes([dig_p[4], dig_p[5]]),
            i16::from_le_bytes([dig_p[6], dig_p[7]]),
            i16::from_le_bytes([dig_p[8], dig_p[9]]),
            i16::from_le_bytes([dig_p[10], dig_p[11]]),
            i16::from_le_bytes([dig_p[12], dig_p[13]]),
            i16::from_le_bytes([dig_p[14], dig_p[15]]),
            i16::from_le_bytes([dig_p[16], dig_p[17]]),
        ));
        Ok(())
    }

    fn load_humidity_data(&mut self) -> Result<(), I2C::Error> {
        let mut dig_h_a: [u8; 1] = [0; 1];
        self.i2c.write_read(self.address, &[0xA1], &mut dig_h_a)?;

        let mut dig_h_b: [u8; 7] = [0; 7];
        self.i2c.write_read(self.address, &[DIG_H], &mut dig_h_b)?;

        self.humidity_data = Some((
            dig_h_a[0],
            i16::from_le_bytes([dig_h_b[0], dig_h_b[1]]),
            dig_h_b[2],
            (i16::from(dig_h_b[3]) << 4_i32) | (i16::from(dig_h_b[4]) & 0x0f),
            ((i16::from(dig_h_b[4]) & 0xf0) >> 4_i32) | (i16::from(dig_h_b[5]) << 4_i32),
            i8::from_be_bytes([dig_h_b[6]]),
        ));
        Ok(())
    }

    pub fn celsius(&mut self) -> Result<i32, I2C::Error> {
        let mut temperature_data: [u8; 3] = [0; 3];
        self.i2c
            .write_read(self.address, &[REG_TEMP], &mut temperature_data)?;
        Ok(self.compensate_t(
            i32::from_be_bytes([
                0,
                temperature_data[0],
                temperature_data[1],
                temperature_data[2],
            ]) >> 4_i32,
        ) / 100_i32)
    }

    pub fn pascal(&mut self) -> Result<u32, I2C::Error> {
        let mut pressure_data: [u8; 3] = [0; 3];
        self.i2c
            .write_read(self.address, &[REG_PRESS], &mut pressure_data)?;

        Ok(self.compensate_p(
            i32::from_be_bytes([0, pressure_data[0], pressure_data[1], pressure_data[2]]) >> 4_i32,
        ) / 256_u32)
    }

    pub fn relative(&mut self) -> Result<u32, I2C::Error> {
        let mut humidity_data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_HUM], &mut humidity_data)?;

        Ok(self.compensate_h(i32::from_be_bytes([
            0,
            0,
            humidity_data[0],
            humidity_data[1],
        ])) / 1024_u32)
    }

    pub fn read_raw(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, &[0xF4, 169])?;
        Ok(())
    }

    fn compensate_t(&mut self, adc_t: i32) -> i32 {
        let var_1: i32 = (((adc_t >> 3) - ((i32::from(self.temperature_data.unwrap().0)) << 1))
            * i32::from(self.temperature_data.unwrap().1))
            >> 11_i32;
        let var_2: i32 = (((((adc_t >> 4_i32) - (i32::from(self.temperature_data.unwrap().0)))
            * ((adc_t >> 4_i32) - (i32::from(self.temperature_data.unwrap().0))))
            >> 12)
            * i32::from(self.temperature_data.unwrap().2))
            >> 14;
        let t_fine = var_1 + var_2;
        self.t_fine = Some(t_fine);
        (t_fine * 5_i32 + 128_i32) >> 8_i32
    }

    fn compensate_p(&mut self, adc_p: i32) -> u32 {
        if self.t_fine.is_none() {
            self.celsius().unwrap();
        }
        let mut var_1: i64 = i64::from(self.t_fine.unwrap()) - 128_000_i64;
        let mut var_2: i64 = var_1 * var_1 * i64::from(self.pressure_data.unwrap().5);
        var_2 += (var_1 * i64::from(self.pressure_data.unwrap().4)) << 17_i32;
        var_2 += i64::from(self.pressure_data.unwrap().3) << 35_i32;
        var_1 = ((var_1 * var_1 * i64::from(self.pressure_data.unwrap().2)) >> 8_i32)
            + ((var_1 * i64::from(self.pressure_data.unwrap().1)) << 12_i32);
        var_1 = (((1 << 47_i32) + var_1) * i64::from(self.pressure_data.unwrap().0)) >> 33_i32;
        let mut p: i64 = 0x0010_0000_i64 - i64::from(adc_p);
        p = (((p << 31_i32) - var_2) * 3125) / NonZeroI64::new(var_1).unwrap().get();
        var_1 =
            (i64::from(self.pressure_data.unwrap().8) * ((p >> 13_i32) * (p >> 13_i32))) >> 25_i32;
        var_2 = (i64::from(self.pressure_data.unwrap().7) * p) >> 19_i32;
        p = ((p + var_1 + var_2) >> 8_i32) + (i64::from(self.pressure_data.unwrap().6) << 4_i32);
        u32(p).unwrap()
    }

    fn compensate_h(&mut self, adc_h: i32) -> u32 {
        if self.t_fine.is_none() {
            self.celsius().unwrap();
        }
        let mut v_x1_u32r: i32 = self.t_fine.unwrap() - 76800_i32;
        v_x1_u32r = ((((adc_h << 14_i32)
            - (i32::from(self.humidity_data.unwrap().3) << 20_i32)
            - (i32::from(self.humidity_data.unwrap().4) * v_x1_u32r))
            + (0x4000_i32))
            >> 15_i32)
            * (((((((v_x1_u32r * i32::from(self.humidity_data.unwrap().5)) >> 10_i32)
                * (((v_x1_u32r * i32::from(self.humidity_data.unwrap().2)) >> 11_i32)
                    + (0x8000_i32)))
                >> 10_i32)
                + (0x0020_0000_i32))
                * i32::from(self.humidity_data.unwrap().1)
                + 8192_i32)
                >> 14_i32);
        v_x1_u32r = v_x1_u32r
            - (((((v_x1_u32r >> 15_i32) * (v_x1_u32r >> 15_i32)) >> 7_i32)
                * i32::from(self.humidity_data.unwrap().0))
                >> 4_i32);
        u32(v_x1_u32r.clamp(0_i32, 419_430_400_i32) >> 12_i32).unwrap()
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use crate::Driver;
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p2::P2;

    #[test]
    pub fn test_compensate_t() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: Some((28834, 26639, 50)),
            pressure_data: None,
            humidity_data: None,
            t_fine: None,
        };
        assert_eq!(p2.compensate_t(0x0008_0020), 2_000_i32);
        assert_eq!(p2.t_fine, Some(102_404_i32));
        i2c_clone.done();
    }

    #[test]
    pub fn test_compensate_p() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: None,
            pressure_data: Some((36219, -10603, 3024, 5679, 24, -7, 9900, -10230, 4285)),
            humidity_data: None,
            t_fine: Some(102_404_i32),
        };

        assert_eq!(p2.compensate_p(0x0005_9386), 25_939_210);
        i2c_clone.done();
    }

    #[test]
    pub fn test_compensate_h() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: None,
            pressure_data: None,
            humidity_data: Some((75, 374, 0, 290, 50, 30)),
            t_fine: Some(102_404_i32),
        };

        assert_eq!(p2.compensate_h(0x6ae4), 51204);
        i2c_clone.done();
    }

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write_read(0x77, vec![0x88], vec![162, 112, 15, 104, 50, 0]),
            I2cTransaction::write_read(
                0x77,
                vec![0x8E],
                vec![
                    123, 141, 149, 214, 208, 11, 47, 22, 24, 0, 249, 255, 172, 38, 10, 216, 189, 16,
                ],
            ),
            I2cTransaction::write_read(0x77, vec![0xA1], vec![75]),
            I2cTransaction::write_read(0x77, vec![0xE1], vec![118, 1, 0, 18, 34, 3, 30]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let p2 = P2::new(i2c, 0x77).unwrap().init().unwrap();
        assert_eq!(p2.temperature_data, Some((28834, 26639, 50)));
        assert_eq!(
            p2.pressure_data,
            Some((36219, -10603, 3024, 5679, 24, -7, 9900, -10230, 4285))
        );
        assert_eq!(p2.humidity_data, Some((75, 374, 0, 290, 50, 30)));

        i2c_clone.done();
    }

    #[test]
    pub fn load_temperature_data() {
        let expectations = [I2cTransaction::write_read(
            0x77,
            vec![0x88],
            vec![162, 112, 15, 104, 50, 0],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: None,
            pressure_data: None,
            humidity_data: None,
            t_fine: None,
        };
        p2.load_temperature_data().unwrap();
        assert_eq!(p2.temperature_data, Some((28834, 26639, 50)));
        assert_eq!(p2.pressure_data, None);
        assert_eq!(p2.humidity_data, None);

        i2c_clone.done();
    }

    #[test]
    pub fn load_pressure_data() {
        let expectations = [I2cTransaction::write_read(
            0x77,
            vec![0x8E],
            vec![
                123, 141, 149, 214, 208, 11, 47, 22, 24, 0, 249, 255, 172, 38, 10, 216, 189, 16,
            ],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: None,
            pressure_data: None,
            humidity_data: None,
            t_fine: None,
        };
        p2.load_pressure_data().unwrap();
        assert_eq!(
            p2.pressure_data,
            Some((36219, -10603, 3024, 5679, 24, -7, 9900, -10230, 4285))
        );
        assert_eq!(p2.temperature_data, None);
        assert_eq!(p2.humidity_data, None);

        i2c_clone.done();
    }

    #[test]
    pub fn load_humidity_data() {
        let expectations = [
            I2cTransaction::write_read(0x77, vec![0xA1], vec![75]),
            I2cTransaction::write_read(0x77, vec![0xE1], vec![118, 1, 0, 18, 34, 3, 30]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: None,
            pressure_data: None,
            humidity_data: None,
            t_fine: None,
        };
        p2.load_humidity_data().unwrap();
        assert_eq!(p2.humidity_data, Some((75, 374, 0, 290, 50, 30)));
        assert_eq!(p2.temperature_data, None);
        assert_eq!(p2.pressure_data, None);

        i2c_clone.done();
    }

    #[test]
    pub fn celsius() {
        let expectations = [I2cTransaction::write_read(
            0x77,
            vec![0xFA],
            vec![129, 145, 0],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: Some((28834, 26639, 50)),
            pressure_data: None,
            humidity_data: None,
            t_fine: Some(112_786_i32),
        };
        assert_eq!(p2.celsius().unwrap(), 22_i32);

        i2c_clone.done();
    }

    #[test]
    pub fn pascal() {
        let expectations = [I2cTransaction::write_read(
            0x77,
            vec![0xF7],
            vec![88, 169, 128],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: None,
            pressure_data: Some((36219, -10603, 3024, 5679, 24, -7, 9900, -10230, 4285)),
            humidity_data: None,
            t_fine: Some(111_446_i32),
        };
        assert_eq!(p2.pascal().unwrap(), 102_003);

        i2c_clone.done();
    }

    #[test]
    pub fn relative() {
        let expectations = [I2cTransaction::write_read(0x77, vec![0xFD], vec![128, 0])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x77,
            temperature_data: None,
            pressure_data: None,
            humidity_data: Some((75, 374, 0, 290, 50, 30)),
            t_fine: Some(111_902_i32),
        };
        assert_eq!(p2.relative().unwrap(), 80);

        i2c_clone.done();
    }
}
