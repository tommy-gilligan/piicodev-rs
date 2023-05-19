#![doc = include_str!("../README.md")]
#![no_std]
#![feature(lint_reasons)]

use embedded_hal::i2c::I2c;
use mfrc522::comm::blocking::i2c::I2cInterface;
use mfrc522::{Initialized, Mfrc522};

pub struct P16<I2C: I2c> {
    #[allow(dead_code)]
    mfrc522: Mfrc522<I2cInterface<I2C>, Initialized>,
}

impl<I2C: I2c> P16<I2C> {
    /// # Errors
    pub fn new(i2c: I2C, address: u8) -> Result<Self, mfrc522::error::Error<I2C::Error>> {
        Ok(Self {
            mfrc522: Mfrc522::new(I2cInterface::new(i2c, address)).init()?,
        })
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

    use crate::P16;

    #[test]
    pub fn new() {
        let expectations = [
            // reset
            I2cTransaction::write(0x2C, vec![0x01, 0x0F]),
            // wait for MFRC522 to be ready
            I2cTransaction::write_read(0x2C, vec![0x01], vec![0xff]),
            I2cTransaction::write_read(0x2C, vec![0x01], vec![0x00]),
            // TxModeReg
            I2cTransaction::write(0x2C, vec![0x12, 0x00]),
            // RxModeReg
            I2cTransaction::write(0x2C, vec![0x13, 0x00]),
            // Reset ModWidthReg to default value
            I2cTransaction::write(0x2C, vec![0x24, 0x26]),
            // Configure the timer, so we can get a timeout if something goes wrong
            // when communicating with a PICC:
            // - Set timer to start automatically at the end of the transmission
            I2cTransaction::write(0x2C, vec![0x2A, 0x80]),
            // - Configure the prescaler to determine the timer frequency:
            //   f_timer = 13.56 MHz / (2 * TPreScaler + 1)
            //   so for 40kHz frequency (25μs period), TPreScaler = 0x0A9
            I2cTransaction::write(0x2C, vec![0x2B, 0xA9]),
            // - Set the reload value to determine the timeout
            //   for a 25ms timeout, we need a value of 1000 = 0x3E8
            I2cTransaction::write(0x2C, vec![0x2C, 0x03]),
            I2cTransaction::write(0x2C, vec![0x2D, 0xE8]),
            I2cTransaction::write(0x2C, vec![0x15, 0x40]),
            // Set preset value of CRC coprocessor according to ISO 14443-3 part 6.2.4
            I2cTransaction::write(0x2C, vec![0x11, (0x3f & (!0b11)) | 0b01]),
            // Enable antenna
            I2cTransaction::write_read(0x2C, vec![0x14], vec![0x11]),
            I2cTransaction::write(0x2C, vec![0x14, 0x13]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P16::new(i2c, 0x2C).unwrap();

        i2c_clone.done();
    }
}
