//! # Unofficial Rust Driver for PiicoDev Pressure Sensor
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Pressure-Sensor-MS5637/tree/7a55775b9c01417b9002f38384aa5bc11ea58a77
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-MS5637-MicroPython-Module/tree/47c7c30d65ee9c189202e949030edcd816f4bfa7
//! [Official Product Site]: https://piico.dev/p11
//! [Datasheet]: https://www.te.com/commerce/DocumentDelivery/DDEController?Action=showdoc&DocId=Data+Sheet%7FMS5637-02BA03%7FB1%7Fpdf%7FEnglish%7FENG_DS_MS5637-02BA03_B1.pdf%7FCAT-BLPS0037

use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::I2c;
use num_enum::IntoPrimitive;

pub struct P11<I2C, DELAY> {
    i2c: I2C,
    address: u8,
    delay: DELAY,
    coefficient_valid: bool,
    eeprom_coefficient: [u16; 7],
}

const ADC_READ: u8 = 0x00;
const SOFT_RESET: u8 = 0x1E;

const MS5637_PROM_ADDR_0: u8 = 0xA0;
const MS5637_PROM_ADDR_1: u8 = 0xA2;
const MS5637_PROM_ADDR_2: u8 = 0xA4;
const MS5637_PROM_ADDR_3: u8 = 0xA6;
const MS5637_PROM_ADDR_4: u8 = 0xA8;
const MS5637_PROM_ADDR_5: u8 = 0xAA;
const MS5637_PROM_ADDR_6: u8 = 0xAC;

const RESOLUTION_OSR_8192: u8 = 5;

const MS5637_PRESSURE_SENSITIVITY_INDEX: usize = 1;
const MS5637_PRESSURE_OFFSET_INDEX: usize = 2;
const MS5637_TEMP_COEFF_OF_PRESSURE_SENSITIVITY_INDEX: usize = 3;
const MS5637_TEMP_COEFF_OF_PRESSURE_OFFSET_INDEX: usize = 4;
const MS5637_REFERENCE_TEMPERATURE_INDEX: usize = 5;
const MS5637_TEMP_COEFF_OF_TEMPERATURE_INDEX: usize = 6;

#[derive(IntoPrimitive)]
#[repr(u32)]
enum ConversionTime {
    Osr256 = 1,
    Osr512 = 2,
    Osr1024 = 3,
    Osr2048 = 5,
    Osr4096 = 9,
    Osr8192 = 17,
}

fn set_resolution(index: u8) -> (u8, u8, u32, u32) {
    let time: [u32; 6] = [
        ConversionTime::Osr256.into(),
        ConversionTime::Osr512.into(),
        ConversionTime::Osr1024.into(),
        ConversionTime::Osr2048.into(),
        ConversionTime::Osr4096.into(),
        ConversionTime::Osr8192.into(),
    ];
    let time_temp: u32 = time[index as usize];
    let time_pressure: u32 = time[index as usize];

    (
        (index << 1_u8) | 0b0101_0000,
        (index << 1_u8) | 0b0100_0000,
        time_temp,
        time_pressure,
    )
}

use crate::WithDelay;
impl<I2C: I2c, DELAY: DelayUs> WithDelay<I2C, DELAY, I2C::Error> for P11<I2C, DELAY> {
    fn new_inner(i2c: I2C, address: u8, delay: DELAY) -> Self {
        Self {
            i2c,
            address,
            delay,
            coefficient_valid: false,
            eeprom_coefficient: [0; 7],
        }
    }

    fn init_inner(mut self) -> Result<Self, I2C::Error> {
        self.i2c.write(self.address, &[SOFT_RESET])?;
        self.delay.delay_ms(15);
        Ok(self)
    }
}

use rust_decimal::prelude::*;
impl<I2C: I2c, DELAY: DelayUs> P11<I2C, DELAY> {
    fn read_eeprom_coefficient(&mut self, reg: u8) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c.write_read(self.address, &[reg], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    /// # Errors
    fn read_eeprom(&mut self) -> Result<[u16; 7], I2C::Error> {
        let mut coefficients: [u16; 7] = [0; 7];

        for (a, i) in [
            MS5637_PROM_ADDR_0,
            MS5637_PROM_ADDR_1,
            MS5637_PROM_ADDR_2,
            MS5637_PROM_ADDR_3,
            MS5637_PROM_ADDR_4,
            MS5637_PROM_ADDR_5,
            MS5637_PROM_ADDR_6,
        ]
        .into_iter()
        .enumerate()
        {
            coefficients[a] = self.read_eeprom_coefficient(i)?;
        }

        self.coefficient_valid = true;
        Ok(coefficients)
    }

    /// # Errors
    fn conversion_read_adc(&mut self, cmd: u8, time: u32) -> Result<u32, I2C::Error> {
        self.i2c.write(self.address, &[cmd])?;
        self.delay.delay_ms(time);
        let mut data: [u8; 3] = [0; 3];
        // cheat checking error for now
        self.i2c.write_read(self.address, &[ADC_READ], &mut data)?;
        Ok(u32::from_be_bytes([0x00, data[0], data[1], data[2]]))
    }

    /// # Errors
    pub fn read_temperature_and_pressure(
        &mut self,
        res: Option<u8>,
    ) -> Result<(Decimal, Decimal), I2C::Error> {
        if !self.coefficient_valid {
            self.eeprom_coefficient = self.read_eeprom()?;
        }
        let (cmd_temp, cmd_pressure, time_temp, time_pressure) =
            set_resolution(res.unwrap_or(RESOLUTION_OSR_8192));
        let adc_temperature: u32 = self.conversion_read_adc(cmd_temp, time_temp)?;
        let adc_pressure: u32 = self.conversion_read_adc(cmd_pressure, time_pressure)?;
        // Difference between actual and reference temperature = D2 - Tref
        let d_t: i64 = i64::from(adc_temperature)
            - i64::from(self.eeprom_coefficient[MS5637_REFERENCE_TEMPERATURE_INDEX]) * 0x100_i64;
        // Actual temperature = 2000 + dT * TEMPSENS
        let temp: i64 = 2000
            + ((d_t * i64::from(self.eeprom_coefficient[MS5637_TEMP_COEFF_OF_TEMPERATURE_INDEX]))
                >> 23);
        // Second order temperature compensation
        let t2: i64;
        let mut off2: i64;
        let mut sens2: i64;

        if temp < 2000_i64 {
            t2 = (3_i64 * (d_t * d_t)) >> 33_i64;
            off2 = 61_i64 * (temp - 2000_i64) * (temp - 2000_i64) / 16_i64;
            sens2 = 29_i64 * (temp - 2000_i64) * (temp - 2000_i64) / 16_i64;
            if temp < -1500_i64 {
                off2 += 17_i64 * (temp + 1500_i64) * (temp + 1500_i64);
                sens2 += 9_i64 * ((temp + 1500_i64) * (temp + 1500_i64));
            }
        } else {
            t2 = (5_i64 * (d_t * d_t)) >> 38_i64;
            off2 = 0_i64;
            sens2 = 0_i64;
        }

        //  OFF = OFF_T1 + TCO * dT
        let mut off: i64 = (i64::from(self.eeprom_coefficient[MS5637_PRESSURE_OFFSET_INDEX]) << 17)
            + ((i64::from(self.eeprom_coefficient[MS5637_TEMP_COEFF_OF_PRESSURE_OFFSET_INDEX])
                * d_t)
                >> 6);
        off -= off2;
        // Sensitivity at actual temperature = SENS_T1 + TCS * dT
        let mut sens = i64::from(
            u32::from(self.eeprom_coefficient[MS5637_PRESSURE_SENSITIVITY_INDEX]) * 0x10000,
        ) + ((i64::from(
            self.eeprom_coefficient[MS5637_TEMP_COEFF_OF_PRESSURE_SENSITIVITY_INDEX],
        ) * d_t)
            >> 7_i64);
        sens -= sens2;
        //  Temperature compensated pressure = D1 * SENS - OFF
        Ok((
            Decimal::new(temp - t2, 2),
            Decimal::new(
                (((i64::from(adc_pressure) * sens) >> 21_i64) - off) >> 15_i64,
                2,
            ),
        ))
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use crate::WithDelay;
    use rust_decimal::prelude::*;
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p11::{set_resolution, P11};

    #[test]
    pub fn new() {
        let expectations = [I2cTransaction::write(0x76, vec![0x1E])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P11::new(i2c, 0x76, embedded_hal_mock::eh1::delay::NoopDelay {})
            .unwrap()
            .init()
            .unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn read_eeprom() {
        let expectations = [
            I2cTransaction::write_read(0x76, vec![0xA0], vec![0x02, 0x01]),
            I2cTransaction::write_read(0x76, vec![0xA2], vec![0x03, 0x02]),
            I2cTransaction::write_read(0x76, vec![0xA4], vec![0x04, 0x03]),
            I2cTransaction::write_read(0x76, vec![0xA6], vec![0x05, 0x04]),
            I2cTransaction::write_read(0x76, vec![0xA8], vec![0x06, 0x05]),
            I2cTransaction::write_read(0x76, vec![0xAA], vec![0x07, 0x06]),
            I2cTransaction::write_read(0x76, vec![0xAC], vec![0x08, 0x07]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p11 = P11 {
            i2c,
            address: 0x76,
            delay: embedded_hal_mock::eh1::delay::NoopDelay {},
            coefficient_valid: false,
            eeprom_coefficient: [0; 7],
        };

        assert_eq!(
            p11.read_eeprom().unwrap(),
            [0x0201, 0x0302, 0x0403, 0x0504, 0x0605, 0x0706, 0x0807]
        );
        i2c_clone.done();
    }

    #[test]
    pub fn set_resolution_test() {
        assert_eq!(set_resolution(5), (90, 74, 17, 17));
    }

    #[test]
    pub fn conversion_read_adc() {
        let expectations = [
            I2cTransaction::write(0x76, vec![90]),
            I2cTransaction::write_read(0x76, vec![0], vec![0x08, 0x07, 0x06]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p11 = P11 {
            i2c,
            address: 0x76,
            delay: embedded_hal_mock::eh1::delay::NoopDelay {},
            coefficient_valid: false,
            eeprom_coefficient: [0; 7],
        };

        assert_eq!(p11.conversion_read_adc(90, 0).unwrap(), 0x0008_0706);
        i2c_clone.done();
    }

    #[test]
    pub fn read_temperature_and_pressure_no_coefficients() {
        let expectations = [
            I2cTransaction::write_read(0x76, vec![0xA0], vec![0x02, 0x01]),
            I2cTransaction::write_read(0x76, vec![0xA2], vec![0x03, 0x02]),
            I2cTransaction::write_read(0x76, vec![0xA4], vec![0x04, 0x03]),
            I2cTransaction::write_read(0x76, vec![0xA6], vec![0x05, 0x04]),
            I2cTransaction::write_read(0x76, vec![0xA8], vec![0x06, 0x05]),
            I2cTransaction::write_read(0x76, vec![0xAA], vec![0x07, 0x06]),
            I2cTransaction::write_read(0x76, vec![0xAC], vec![0x08, 0x07]),
            I2cTransaction::write(0x76, vec![90]),
            I2cTransaction::write_read(0x76, vec![0x00], vec![0xF0, 0x00, 0x00]),
            I2cTransaction::write(0x76, vec![74]),
            I2cTransaction::write_read(0x76, vec![0x00], vec![0x78, 0x77, 0x76]),
        ];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p11 = P11 {
            i2c,
            address: 0x76,
            delay: embedded_hal_mock::eh1::delay::NoopDelay {},
            coefficient_valid: false,
            eeprom_coefficient: [0; 7],
        };

        assert_eq!(
            p11.read_temperature_and_pressure(None).unwrap(),
            (Decimal::new(1500, 2), Decimal::new(8066, 2),)
        );
        i2c_clone.done();
    }

    #[test]
    pub fn read_temperature_and_pressure_with_coefficients() {
        let expectations = [
            I2cTransaction::write(0x76, vec![90]),
            I2cTransaction::write_read(0x76, vec![0x00], vec![0xF0, 0x00, 0x00]),
            I2cTransaction::write(0x76, vec![74]),
            I2cTransaction::write_read(0x76, vec![0x00], vec![0x78, 0x77, 0x76]),
        ];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p11 = P11 {
            i2c,
            address: 0x76,
            delay: embedded_hal_mock::eh1::delay::NoopDelay {},
            coefficient_valid: true,
            // exercises temp < 2000 branch
            eeprom_coefficient: [
                0b1111_1111_1110_1001,
                0b1111_1111_1110_1001,
                0b1111_1111_1110_1001,
                0b1111_1111_1110_1001,
                0b1111_1111_1110_1001,
                0b1111_1111_1110_1001,
                0b1111_1111_1110_1001,
            ],
        };

        assert_eq!(
            p11.read_temperature_and_pressure(None).unwrap(),
            (Decimal::new(-6523, 2), Decimal::new(185_262, 2),)
        );
        i2c_clone.done();
    }
}
