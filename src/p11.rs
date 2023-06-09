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
use measurements::{Pressure, Temperature};

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

const MS5637_START_TEMPERATURE_ADC_CONVERSION: u8 = 0x50;
const MS5637_START_PRESSURE_ADC_CONVERSION: u8 = 0x40;
const MS5637_CONVERSION_OSR_MASK: u8 = 0x0F;

#[expect(dead_code)]
const RESOLUTION_OSR_256: u8 = 0;
#[expect(dead_code)]
const RESOLUTION_OSR_512: u8 = 1;
#[expect(dead_code)]
const RESOLUTION_OSR_1024: u8 = 2;
#[expect(dead_code)]
const RESOLUTION_OSR_2048: u8 = 3;
#[expect(dead_code)]
const RESOLUTION_OSR_4096: u8 = 4;
const RESOLUTION_OSR_8192: u8 = 5;

const MS5637_PRESSURE_SENSITIVITY_INDEX: u8 = 1;
const MS5637_PRESSURE_OFFSET_INDEX: u8 = 2;
const MS5637_TEMP_COEFF_OF_PRESSURE_SENSITIVITY_INDEX: u8 = 3;
const MS5637_TEMP_COEFF_OF_PRESSURE_OFFSET_INDEX: u8 = 4;
const MS5637_REFERENCE_TEMPERATURE_INDEX: u8 = 5;
const MS5637_TEMP_COEFF_OF_TEMPERATURE_INDEX: u8 = 6;

// 0.001
const MS5637_CONV_TIME_OSR_256: u32 = 1;
// 0.002
const MS5637_CONV_TIME_OSR_512: u32 = 2;
// 0.003
const MS5637_CONV_TIME_OSR_1024: u32 = 3;
// 0.005
const MS5637_CONV_TIME_OSR_2048: u32 = 5;
// 0.009
const MS5637_CONV_TIME_OSR_4096: u32 = 9;
// 0.017
const MS5637_CONV_TIME_OSR_8192: u32 = 17;

const fn set_resolution(res: u8) -> (u8, u8, u32, u32) {
    let time: [u32; 6] = [
        MS5637_CONV_TIME_OSR_256,
        MS5637_CONV_TIME_OSR_512,
        MS5637_CONV_TIME_OSR_1024,
        MS5637_CONV_TIME_OSR_2048,
        MS5637_CONV_TIME_OSR_4096,
        MS5637_CONV_TIME_OSR_8192,
    ];
    let cmd_temp: u8 = (res * 2) | MS5637_START_TEMPERATURE_ADC_CONVERSION;
    let time_temp: u32 = time[((cmd_temp & MS5637_CONVERSION_OSR_MASK) / 2) as usize];
    let cmd_pressure: u8 = (res * 2) | MS5637_START_PRESSURE_ADC_CONVERSION;
    let time_pressure: u32 = time[((cmd_temp & MS5637_CONVERSION_OSR_MASK) / 2) as usize];

    (cmd_temp, cmd_pressure, time_temp, time_pressure)
}

use crate::WithDelay;
impl<I2C: I2c, DELAY: DelayUs> WithDelay<I2C, DELAY> for P11<I2C, DELAY> {
    fn new(i2c: I2C, address: u8, delay: DELAY) -> Self {
        Self {
            i2c,
            address,
            delay,
            coefficient_valid: false,
            eeprom_coefficient: [0; 7],
        }
    }
}

impl<I2C: I2c, DELAY: DelayUs> P11<I2C, DELAY> {
    pub fn init(mut self) -> Result<Self, I2C::Error> {
        self.i2c.write(self.address, &[SOFT_RESET])?;
        self.delay.delay_ms(15);
        Ok(self)
    }

    /// # Errors
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
    ) -> Result<(Temperature, Pressure), I2C::Error> {
        if !self.coefficient_valid {
            self.eeprom_coefficient = self.read_eeprom()?;
        }
        let (cmd_temp, cmd_pressure, time_temp, time_pressure) =
            set_resolution(res.unwrap_or(RESOLUTION_OSR_8192));
        let adc_temperature: u32 = self.conversion_read_adc(cmd_temp, time_temp)?;
        let adc_pressure: u32 = self.conversion_read_adc(cmd_pressure, time_pressure)?;
        // Difference between actual and reference temperature = D2 - Tref
        #[expect(clippy::cast_possible_wrap)]
        let d_t: i64 = i64::from(adc_temperature)
            - ((u64::from(self.eeprom_coefficient[MS5637_REFERENCE_TEMPERATURE_INDEX as usize])
                * 0x100_u64) as i64);
        // Actual temperature = 2000 + dT * TEMPSENS
        let temp: i64 = 2000
            + ((d_t
                * i64::from(
                    self.eeprom_coefficient[MS5637_TEMP_COEFF_OF_TEMPERATURE_INDEX as usize],
                ))
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
        let mut off: i64 =
            (i64::from(self.eeprom_coefficient[MS5637_PRESSURE_OFFSET_INDEX as usize]) << 17)
                + ((i64::from(
                    self.eeprom_coefficient[MS5637_TEMP_COEFF_OF_PRESSURE_OFFSET_INDEX as usize],
                ) * d_t)
                    >> 6);
        off -= off2;
        // Sensitivity at actual temperature = SENS_T1 + TCS * dT
        let mut sens = i64::from(
            u32::from(self.eeprom_coefficient[MS5637_PRESSURE_SENSITIVITY_INDEX as usize])
                * 0x10000,
        ) + ((i64::from(
            self.eeprom_coefficient[MS5637_TEMP_COEFF_OF_PRESSURE_SENSITIVITY_INDEX as usize],
        ) * d_t)
            >> 7_i64);
        sens -= sens2;
        //  Temperature compensated pressure = D1 * SENS - OFF
        let p = (((i64::from(adc_pressure) * sens) >> 21_i64) - off) >> 15_i64;
        #[expect(clippy::cast_precision_loss)]
        let temperature = (temp - t2) as f64 / 100.0_f64;
        #[expect(clippy::cast_precision_loss)]
        let pressure = p as f64 / 100.0_f64;

        Ok((
            Temperature::from_celsius(temperature),
            Pressure::from_hectopascals(pressure),
        ))
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use crate::WithDelay;
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::delay::MockNoop;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use measurements::{Pressure, Temperature};

    use crate::p11::{set_resolution, P11};

    #[test]
    pub fn new() {
        let expectations = [I2cTransaction::write(0x76, vec![0x1E])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P11::new(i2c, 0x76, MockNoop {}).init().unwrap();
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
            delay: MockNoop {},
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
            delay: MockNoop {},
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
            delay: MockNoop {},
            coefficient_valid: false,
            eeprom_coefficient: [0; 7],
        };

        assert_eq!(
            p11.read_temperature_and_pressure(None).unwrap(),
            (
                Temperature::from_kelvin(288.15),
                Pressure::from_pascals(8066.0),
            )
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
            delay: MockNoop {},
            coefficient_valid: true,
            // exercises temp < 2000 branch
            eeprom_coefficient: [
                -23_i16 as u16,
                -23_i16 as u16,
                -23_i16 as u16,
                -23_i16 as u16,
                -23_i16 as u16,
                -23_i16 as u16,
                -23_i16 as u16,
            ],
        };

        assert_eq!(
            p11.read_temperature_and_pressure(None).unwrap(),
            (
                Temperature::from_kelvin(207.919_999_999_999_96),
                Pressure::from_pascals(185_262.0),
            )
        );
        i2c_clone.done();
    }
}
