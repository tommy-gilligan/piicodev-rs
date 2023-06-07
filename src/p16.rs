//! # Unofficial Rust Driver for PiicoDev RFID Module
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-RFID-Module
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-RFID-MicroPython-Module
//! [Official Product Site]: https://piico.dev/p16
//! [Datasheet]: https://github.com/CoreElectronics/CE-PiicoDev-RFID-Module/raw/main/Documents/MFRC522.pdf
use embedded_hal::i2c::I2c;
use mfrc522::comm::blocking::i2c::I2cInterface;
pub use mfrc522::{GenericUid, Uid};
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

    pub fn read_tag_id(&mut self) -> Result<Uid, mfrc522::error::Error<I2C::Error>> {
        let atqa = self.mfrc522.reqa()?;
        let uid = self.mfrc522.select(&atqa)?;
        Ok(uid)
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p16::{GenericUid, Uid, P16};
    use mfrc522::comm::blocking::i2c::I2cInterface;
    use mfrc522::{Initialized, Mfrc522};

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

    #[test]
    pub fn read_tag_id_ntag() {
        let expectations = [
            // reqa
            I2cTransaction::write(0x2C, vec![0x01, 0x0F]),
            I2cTransaction::write_read(0x2C, vec![0x01], vec![0xff]),
            I2cTransaction::write_read(0x2C, vec![0x01], vec![0x00]),
            I2cTransaction::write(0x2C, vec![0x12, 0x00]),
            I2cTransaction::write(0x2C, vec![0x13, 0x00]),
            I2cTransaction::write(0x2C, vec![0x24, 0x26]),
            I2cTransaction::write(0x2C, vec![0x2A, 0x80]),
            I2cTransaction::write(0x2C, vec![0x2B, 0xA9]),
            I2cTransaction::write(0x2C, vec![0x2C, 0x03]),
            I2cTransaction::write(0x2C, vec![0x2D, 0xE8]),
            I2cTransaction::write(0x2C, vec![0x15, 0x40]),
            I2cTransaction::write(0x2C, vec![0x11, (0x3f & (!0b11)) | 0b01]),
            I2cTransaction::write_read(0x2C, vec![0x14], vec![0x11]),
            I2cTransaction::write(0x2C, vec![0x14, 0x13]),
            I2cTransaction::write(0x2C, vec![0x01, 0x00]),
            I2cTransaction::write(0x2C, vec![0x04, 0x7F]),
            I2cTransaction::write(0x2C, vec![0x0A, 0x80]),
            I2cTransaction::transaction_start(0x2c),
            I2cTransaction::write(0x2C, [0x09].to_vec()),
            I2cTransaction::write(0x2C, [0x26].to_vec()),
            I2cTransaction::transaction_end(0x2c),
            I2cTransaction::write(0x2C, vec![0x01, 0x0C]),
            I2cTransaction::write(0x2C, vec![0x0D, 0x87]),
            I2cTransaction::write_read(0x2C, vec![0x04], vec![0x04]),
            I2cTransaction::write_read(0x2C, vec![0x04], vec![0x02]),
            I2cTransaction::write_read(0x2C, vec![0x06], vec![0x00]),
            I2cTransaction::write_read(0x2C, vec![0x0A], vec![0x02]),
            I2cTransaction::write_read(0x2C, vec![0x09], vec![0xA8, 0xCD]),
            I2cTransaction::write_read(0x2C, vec![0x0C], vec![0x00]),
            // select
            I2cTransaction::write_read(0x2C, vec![0x0E], vec![0x99]),
            I2cTransaction::write(0x2C, vec![0x0E, 0x19]),
            //    transceive
            I2cTransaction::write(0x2C, vec![0x01, 0x00]),
            I2cTransaction::write(0x2C, vec![0x04, 0x7F]),
            I2cTransaction::write(0x2C, vec![0x0A, 0x80]),
            I2cTransaction::transaction_start(0x2c),
            I2cTransaction::write(0x2C, [0x09].to_vec()),
            I2cTransaction::write(0x2C, [0x93, 32].to_vec()),
            I2cTransaction::transaction_end(0x2c),
            I2cTransaction::write(0x2C, vec![0x01, 0x0C]),
            I2cTransaction::write(0x2C, vec![0x0D, 0x80]),
            I2cTransaction::write_read(0x2C, vec![0x04], vec![0x04]),
            I2cTransaction::write_read(0x2C, vec![0x04], vec![0x02]),
            I2cTransaction::write_read(0x2C, vec![0x06], vec![0x00]),
            I2cTransaction::write_read(0x2C, vec![0x0A], vec![0x05]),
            I2cTransaction::write_read(0x2C, vec![0x09], vec![0x01, 0x23, 0x45, 0x67, 0x89]),
            I2cTransaction::write_read(0x2C, vec![0x0C], vec![0x00]),
            //    calculate crc
            I2cTransaction::write(0x2C, vec![0x01, 0x00]),
            I2cTransaction::write(0x2C, vec![0x05, 0x04]),
            I2cTransaction::write(0x2C, vec![0x0A, 0x80]),
            I2cTransaction::transaction_start(0x2c),
            I2cTransaction::write(0x2C, [0x09].to_vec()),
            I2cTransaction::write(0x2C, [0x93, 0x70, 0x01, 0x23, 0x45, 0x67, 0x00].to_vec()),
            I2cTransaction::transaction_end(0x2c),
            I2cTransaction::write(0x2C, vec![0x01, 0x03]),
            I2cTransaction::write_read(0x2C, vec![0x05], vec![0x04]),
            I2cTransaction::write(0x2C, vec![0x01, 0x00]),
            I2cTransaction::write_read(0x2C, vec![0x22], vec![0xab]),
            I2cTransaction::write_read(0x2C, vec![0x21], vec![0xef]),
            //    transceive
            I2cTransaction::write(0x2C, vec![0x01, 0x00]),
            I2cTransaction::write(0x2C, vec![0x04, 0x7F]),
            I2cTransaction::write(0x2C, vec![0x0A, 0x80]),
            I2cTransaction::transaction_start(0x2c),
            I2cTransaction::write(0x2C, [0x09].to_vec()),
            I2cTransaction::write(
                0x2C,
                [0x93, 0x70, 0x01, 0x23, 0x45, 0x67, 0x00, 0xab, 0xef].to_vec(),
            ),
            I2cTransaction::transaction_end(0x2c),
            I2cTransaction::write(0x2C, vec![0x01, 0x0C]),
            I2cTransaction::write(0x2C, vec![0x0D, 0x80]),
            I2cTransaction::write_read(0x2C, vec![0x04], vec![0x02]),
            I2cTransaction::write_read(0x2C, vec![0x06], vec![0x00]),
            I2cTransaction::write_read(0x2C, vec![0x0A], vec![0x03]),
            I2cTransaction::write_read(0x2C, vec![0x09], vec![0x20, 0x4d, 0x01]),
            I2cTransaction::write_read(0x2C, vec![0x0C], vec![0x00]),
            //    calculate crc
            I2cTransaction::write(0x2C, vec![0x01, 0x00]),
            I2cTransaction::write(0x2C, vec![0x05, 0x04]),
            I2cTransaction::write(0x2C, vec![0x0A, 0x80]),
            I2cTransaction::transaction_start(0x2c),
            I2cTransaction::write(0x2C, [0x09].to_vec()),
            I2cTransaction::write(0x2C, [0x20].to_vec()),
            I2cTransaction::transaction_end(0x2c),
            I2cTransaction::write(0x2C, vec![0x01, 0x03]),
            I2cTransaction::write_read(0x2C, vec![0x05], vec![0x04]),
            I2cTransaction::write(0x2C, vec![0x01, 0x00]),
            I2cTransaction::write_read(0x2C, vec![0x22], vec![0x4d]),
            I2cTransaction::write_read(0x2C, vec![0x21], vec![0x01]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p16 = P16 {
            mfrc522: Mfrc522::new(I2cInterface::new(i2c, 0x2C)).init().unwrap(),
        };
        let tag_id = p16.read_tag_id().unwrap();
        assert_eq!(tag_id.as_bytes(), [1, 35, 69, 103]);

        i2c_clone.done();
    }
}
