#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![no_std]

use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::I2c;
use measurements::Distance;

pub struct P7<I2C, DELAY> {
    i2c: I2C,
    address: u8,
    delay: DELAY,
}

const VL51L1X_DEFAULT_CONFIGURATION: [u8; 93] = [
    0x00, 0x2D,
    0x00, // 0x2d : set bit 2 and 5 to 1 for fast plus mode (1MHz I2C), else don't touch */
    0x00, // 0x2e : bit 0 if I2C pulled up at 1.8V, else set bit 0 to 1 (pull up at AVDD) */
    0x00, // 0x2f : bit 0 if GPIO pulled up at 1.8V, else set bit 0 to 1 (pull up at AVDD) */
    0x01, // 0x30 : set bit 4 to 0 for active high interrupt and 1 for active low (bits 3:0 must be 0x1), use SetInterruptPolarity() */
    0x02, // 0x31 : bit 1 = interrupt depending on the polarity, use CheckForDataReady() */
    0x00, // 0x32 : not user-modifiable (NUM)*/
    0x02, // 0x33 : NUM */
    0x08, // 0x34 : NUM */
    0x00, // 0x35 : NUM */
    0x08, // 0x36 : NUM */
    0x10, // 0x37 : NUM */
    0x01, // 0x38 : NUM */
    0x01, // 0x39 : NUM */
    0x00, // 0x3a : NUM */
    0x00, // 0x3b : NUM */
    0x00, // 0x3c : NUM */
    0x00, // 0x3d : NUM */
    0xff, // 0x3e : NUM */
    0x00, // 0x3f : NUM */
    0x0F, // 0x40 : NUM */
    0x00, // 0x41 : NUM */
    0x00, // 0x42 : NUM */
    0x00, // 0x43 : NUM */
    0x00, // 0x44 : NUM */
    0x00, // 0x45 : NUM */
    0x20, // 0x46 : interrupt configuration 0->level low detection, 1-> level high, 2-> Out of window, 3->In window, 0x20-> New sample ready , TBC */
    0x0b, // 0x47 : NUM */
    0x00, // 0x48 : NUM */
    0x00, // 0x49 : NUM */
    0x02, // 0x4a : NUM */
    0x0a, // 0x4b : NUM */
    0x21, // 0x4c : NUM */
    0x00, // 0x4d : NUM */
    0x00, // 0x4e : NUM */
    0x05, // 0x4f : NUM */
    0x00, // 0x50 : NUM */
    0x00, // 0x51 : NUM */
    0x00, // 0x52 : NUM */
    0x00, // 0x53 : NUM */
    0xc8, // 0x54 : NUM */
    0x00, // 0x55 : NUM */
    0x00, // 0x56 : NUM */
    0x38, // 0x57 : NUM */
    0xff, // 0x58 : NUM */
    0x01, // 0x59 : NUM */
    0x00, // 0x5a : NUM */
    0x08, // 0x5b : NUM */
    0x00, // 0x5c : NUM */
    0x00, // 0x5d : NUM */
    0x01, // 0x5e : NUM */
    0xdb, // 0x5f : NUM */
    0x0f, // 0x60 : NUM */
    0x01, // 0x61 : NUM */
    0xf1, // 0x62 : NUM */
    0x0d, // 0x63 : NUM */
    0x01, // 0x64 : Sigma threshold MSB (mm in 14.2 format for MSB+LSB), use SetSigmaThreshold(), default value 90 mm  */
    0x68, // 0x65 : Sigma threshold LSB */
    0x00, // 0x66 : Min count Rate MSB (MCPS in 9.7 format for MSB+LSB), use SetSignalThreshold() */
    0x80, // 0x67 : Min count Rate LSB */
    0x08, // 0x68 : NUM */
    0xb8, // 0x69 : NUM */
    0x00, // 0x6a : NUM */
    0x00, // 0x6b : NUM */
    0x00, // 0x6c : Intermeasurement period MSB, 32 bits register, use SetIntermeasurementInMs() */
    0x00, // 0x6d : Intermeasurement period */
    0x0f, // 0x6e : Intermeasurement period */
    0x89, // 0x6f : Intermeasurement period LSB */
    0x00, // 0x70 : NUM */
    0x00, // 0x71 : NUM */
    0x00, // 0x72 : distance threshold high MSB (in mm, MSB+LSB), use SetD:tanceThreshold() */
    0x00, // 0x73 : distance threshold high LSB */
    0x00, // 0x74 : distance threshold low MSB ( in mm, MSB+LSB), use SetD:tanceThreshold() */
    0x00, // 0x75 : distance threshold low LSB */
    0x00, // 0x76 : NUM */
    0x01, // 0x77 : NUM */
    0x0f, // 0x78 : NUM */
    0x0d, // 0x79 : NUM */
    0x0e, // 0x7a : NUM */
    0x0e, // 0x7b : NUM */
    0x00, // 0x7c : NUM */
    0x00, // 0x7d : NUM */
    0x02, // 0x7e : NUM */
    0xc7, // 0x7f : ROI center, use SetROI() */
    0xff, // 0x80 : XY ROI (X=Width, Y=Height), use SetROI() */
    0x9B, // 0x81 : NUM */
    0x00, // 0x82 : NUM */
    0x00, // 0x83 : NUM */
    0x00, // 0x84 : NUM */
    0x01, // 0x85 : NUM */
    0x01, // 0x86 : clear interrupt, use ClearInterrupt() */
    0x40, // 0x87 : start ranging, use StartRanging() or StopRanging(), If you want an automatic start after VL53L1X_init() call, put 0x40 in location 0x87 */
];

impl<I2C: I2c, DELAY: DelayUs> P7<I2C, DELAY> {
    /// # Errors
    pub fn new(i2c: I2C, address: u8, delay: DELAY) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            address,
            delay,
        };
        res.reset()?;
        res.delay.delay_ms(1);
        res.i2c.write(res.address, &VL51L1X_DEFAULT_CONFIGURATION)?;
        res.delay.delay_ms(100);
        // the API triggers this change in VL53L1_init_and_start_range() once a
        // measurement is started; assumes MM1 and MM2 are disabled
        let mut data: [u8; 2] = [0; 2];
        res.i2c.write_read(res.address, &[0x00, 0x22], &mut data)?;
        data = u16::to_le_bytes(u16::from_le_bytes(data) * 4);
        res.i2c
            .write(res.address, &[0x00, 0x1E, data[0], data[1]])?;
        res.delay.delay_ms(200);
        Ok(res)
    }

    /// # Errors
    pub fn reset(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, &[0x00, 0x00, 0x00])?;
        self.delay.delay_ms(100);
        self.i2c.write(self.address, &[0x00, 0x00, 0x01])?;
        Ok(())
    }

    /// # Errors
    pub fn read(&mut self) -> Result<Distance, I2C::Error> {
        let mut data: [u8; 17] = [0; 17];
        self.i2c
            .write_read(self.address, &[0x00, 0x89], &mut data)?;
        Ok(Distance::from_millimetres(
            u16::from_be_bytes([data[13], data[14]]).into(),
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
    use embedded_hal_mock::delay::MockNoop;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::P7;
    use measurements::Distance;

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x29, vec![0x00, 0x00, 0x00]),
            I2cTransaction::write(0x29, vec![0x00, 0x00, 0x01]),
            I2cTransaction::write(
                0x29,
                vec![
                    0, 45, 0, 0, 0, 1, 2, 0, 2, 8, 0, 8, 16, 1, 1, 0, 0, 0, 0, 255, 0, 15, 0, 0, 0,
                    0, 0, 32, 11, 0, 0, 2, 10, 33, 0, 0, 5, 0, 0, 0, 0, 200, 0, 0, 56, 255, 1, 0,
                    8, 0, 0, 1, 219, 15, 1, 241, 13, 1, 104, 0, 128, 8, 184, 0, 0, 0, 0, 15, 137,
                    0, 0, 0, 0, 0, 0, 0, 1, 15, 13, 14, 14, 0, 0, 2, 199, 255, 155, 0, 0, 0, 1, 1,
                    64,
                ],
            ),
            I2cTransaction::write_read(0x29, vec![0x00, 0x22], vec![0x01, 0x10]),
            I2cTransaction::write(0x29, vec![0x00, 0x1E, 0x04, 0x40]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P7::new(i2c, 0x29, MockNoop {}).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn reset() {
        let expectations = [
            I2cTransaction::write(0x29, vec![0x00, 0x00, 0x00]),
            I2cTransaction::write(0x29, vec![0x00, 0x00, 0x01]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p7 = P7 {
            i2c,
            address: 0x29,
            delay: MockNoop {},
        };
        p7.reset().unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn read() {
        let expectations = [I2cTransaction::write_read(
            0x29,
            vec![0x00, 0x89],
            vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
                0xE8, 0x00, 0x00,
            ],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p7 = P7 {
            i2c,
            address: 0x29,
            delay: MockNoop {},
        };
        assert_eq!(p7.read().unwrap(), Distance::from_millimetres(1000.0));

        i2c_clone.done();
    }
}
