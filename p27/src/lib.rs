#![doc = include_str!("../README.md")]
#![no_std]
#![feature(lint_reasons)]

use core::num::TryFromIntError;
use embedded_hal::{delay::DelayUs, i2c::I2c};
use fugit::{Hertz, RateExtU32};

const REG_WHOAMI: u8 = 0x01;
const REG_FIRM_MAJ: u8 = 0x02;
const REG_FIRM_MIN: u8 = 0x03;
const REG_I2C_ADDRESS: u8 = 0x04;
const REG_LED: u8 = 0x05;

const REG_TX_POWER: u8 = 0x13;
const REG_RFM69_RADIO_STATE: u8 = 0x14;
const REG_RFM69_NODE_ID: u8 = 0x15;
const REG_RFM69_NETWORK_ID: u8 = 0x16;
const REG_RFM69_TO_NODE_ID: u8 = 0x17;
const REG_RFM69_REG: u8 = 0x18;
const REG_RFM69_VALUE: u8 = 0x19;
const REG_RFM69_RESET: u8 = 0x20;
const REG_PAYLOAD_LENGTH: u8 = 0x21;
const REG_PAYLOAD: u8 = 0x22;
const REG_PAYLOAD_NEW: u8 = 0x23;
const REG_PAYLOAD_GO: u8 = 0x24;
const REG_TRANSCEIVER_READY: u8 = 0x25;
const RFM69_REG_BITRATEMSB: u8 = 0x03;
const RFM69_REG_BITRATELSB: u8 = 0x04;
const RFM69_REG_FRFMSB: u8 = 0x07;
const RFM69_REG_FRFMID: u8 = 0x08;
const RFM69_REG_FRFLSB: u8 = 0x09;
const F_STEP: u32 = 61;
const F_XOSC: u32 = 32_000_000;
const DEVICE_ID: u16 = 495;

pub struct P27<I2C, DELAY> {
    i2c: I2C,
    address: u8,
    delay: DELAY,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    TryFromIntError(TryFromIntError),
    I2cError(E),
    ArgumentError,
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Self::I2cError(error)
    }
}

impl<I2C: I2c, DELAY: DelayUs> P27<I2C, DELAY> {
    /// # Errors
    pub fn new(i2c: I2C, address: u8, delay: DELAY) -> Result<Self, Error<I2C::Error>> {
        let mut res = Self {
            i2c,
            address,
            delay,
        };

        res.set_led(true)?;

        res.i2c
            .write(res.address, &[REG_RFM69_NODE_ID | 0x80, 0, 0])?;
        while !(res.transceiver_ready()?) {
            res.delay.delay_ms(10);
        }
        res.i2c
            .write(res.address, &[REG_RFM69_NETWORK_ID | 0x80, 0])?;

        res.set_radio_frequency(922.MHz())?;
        res.set_bit_rate(9_600)?;

        res.set_tx_power(20)?;
        if res.whoami()? != DEVICE_ID {}
        Ok(res)
    }

    /// # Errors
    pub fn send(&mut self, address: u16, data: &[u8]) -> Result<(), Error<I2C::Error>> {
        self.set_destination_radio_address(address)?;
        self.i2c.write(
            self.address,
            &[
                REG_PAYLOAD_LENGTH | 0x80,
                data.len().try_into().map_err(Error::TryFromIntError)?,
            ],
        )?;
        self.delay.delay_ms(5);

        let mut send_data: [u8; 33] = [REG_PAYLOAD | 0x80; 33];
        for (i, _c) in data.iter().enumerate() {
            send_data[i + 1] = data[i];
        }
        self.i2c.write(self.address, &send_data[0..=data.len()])?;
        self.delay.delay_ms(5);

        self.i2c.write(self.address, &[REG_PAYLOAD_GO | 0x80, 1])?;
        Ok(())
    }

    /// # Errors
    pub fn new_payload(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_PAYLOAD_NEW], &mut data)?;
        Ok(data[0] == 1)
    }

    /// # Errors
    pub fn get_destination_radio_address(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c
            .write_read(self.address, &[REG_RFM69_TO_NODE_ID], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    /// # Errors
    pub fn set_destination_radio_address(&mut self, value: u16) -> Result<(), I2C::Error> {
        let data: [u8; 2] = u16::to_be_bytes(value);
        self.i2c.write(
            self.address,
            &[REG_RFM69_TO_NODE_ID | 0x80, data[0], data[1]],
        )?;
        Ok(())
    }

    /// # Errors
    fn set_rfm69_register(&mut self, register: u8, value: u8) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[REG_RFM69_REG | 0x80, register])?;
        self.i2c
            .write(self.address, &[REG_RFM69_VALUE | 0x80, value])?;
        Ok(())
    }

    /// # Errors
    pub fn set_bit_rate(&mut self, bit_rate: u32) -> Result<(), Error<I2C::Error>> {
        if !(1_u32..=F_XOSC).contains(&bit_rate) {
            return Err(Error::ArgumentError);
        }
        let [_, _, msb, lsb] = u32::to_be_bytes(F_XOSC / bit_rate);

        self.delay.delay_ms(10);
        self.set_rfm69_register(RFM69_REG_BITRATEMSB, msb)?;
        self.delay.delay_ms(10);
        self.set_rfm69_register(RFM69_REG_BITRATELSB, lsb)?;
        self.delay.delay_ms(10);
        Ok(())
    }

    /// # Errors
    pub fn set_radio_frequency(&mut self, frequency: Hertz<u32>) -> Result<(), Error<I2C::Error>> {
        if frequency.to_MHz() < 890 || frequency.to_MHz() > 1020 {
            return Err(Error::ArgumentError);
        }
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        let hz = frequency.to_Hz() / F_STEP;

        self.delay.delay_ms(5);
        self.set_rfm69_register(RFM69_REG_FRFMSB, (hz >> 16) as u8)?;
        self.delay.delay_ms(5);
        self.set_rfm69_register(RFM69_REG_FRFMID, (hz >> 8) as u8)?;
        self.delay.delay_ms(5);
        self.set_rfm69_register(RFM69_REG_FRFLSB, hz as u8)?;
        self.delay.delay_ms(5);
        Ok(())
    }

    /// # Errors
    pub fn get_tx_power(&mut self) -> Result<i8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        self.i2c
            .write_read(self.address, &[REG_TX_POWER], &mut data)?;

        #[expect(clippy::cast_possible_wrap)]
        return Ok(data[0] as i8);
    }

    /// # Errors
    pub fn set_tx_power(&mut self, value: i8) -> Result<(), I2C::Error> {
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        #[expect(clippy::cast_sign_loss)]
        self.i2c
            .write(self.address, &[REG_TX_POWER | 0x80, value as u8])?;
        Ok(())
    }

    /// # Errors
    pub fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        if on {
            self.i2c.write(self.address, &[REG_LED | 0x80, 1])?;
        } else {
            self.i2c.write(self.address, &[REG_LED | 0x80, 0])?;
        }
        Ok(())
    }

    /// # Errors
    pub fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut maj: [u8; 2] = [0, 0];
        self.i2c.write_read(self.address, &[REG_WHOAMI], &mut maj)?;
        Ok(u16::from_be_bytes(maj))
    }

    /// # Errors
    pub fn transceiver_ready(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_TRANSCEIVER_READY], &mut data)?;
        Ok(data[0] == 1)
    }

    pub fn receive(&mut self, data: &mut [u8]) -> Result<usize, I2C::Error> {
        if self.new_payload()? {
            let mut payload_length: [u8; 1] = [0];
            self.i2c
                .write_read(self.address, &[REG_PAYLOAD_LENGTH], &mut payload_length)?;
            self.i2c.write_read(
                self.address,
                &[REG_PAYLOAD],
                &mut data[0..(payload_length[0] as usize)],
            )?;
            Ok(payload_length[0] as usize)
        } else {
            Ok(0)
        }
    }

    pub fn enable(&mut self) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[REG_RFM69_RADIO_STATE | 0x80, 1])?;
        self.delay.delay_ms(5);
        Ok(())
    }

    pub fn disable(&mut self) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[REG_RFM69_RADIO_STATE | 0x80, 0])?;
        self.delay.delay_ms(5);
        Ok(())
    }

    pub fn enabled(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_RFM69_RADIO_STATE], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn reset(&mut self) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[REG_RFM69_RESET | 0x80, 0x01])?;
        Ok(())
    }

    pub fn firmware(&mut self) -> Result<(u8, u8), I2C::Error> {
        let mut maj_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_FIRM_MAJ], &mut maj_data)?;
        let mut min_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_FIRM_MIN], &mut min_data)?;
        Ok((maj_data[0], min_data[0]))
    }

    // should somehow destroy instance after call
    pub fn set_address(&mut self, new_address: u8) -> Result<(), Error<I2C::Error>> {
        if !(0x08..=0x77).contains(&new_address) {
            return Err(Error::ArgumentError);
        }
        self.i2c
            .write(self.address, &[REG_I2C_ADDRESS, new_address])?;
        Ok(())
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
#[macro_use]
extern crate std;

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal::i2c::ErrorKind;
    use embedded_hal_mock::delay::MockNoop;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use fugit::RateExtU32;

    use crate::{Error, P27};

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x09, vec![0x85, 0x01]),
            I2cTransaction::write(0x09, vec![0x95, 0x00, 0x00]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x00]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x96, 0x00]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x00]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x98, 0x07]),
            I2cTransaction::write(0x09, vec![0x99, 0xE6]),
            I2cTransaction::write(0x09, vec![0x98, 0x08]),
            I2cTransaction::write(0x09, vec![0x99, 0xA2]),
            I2cTransaction::write(0x09, vec![0x98, 0x09]),
            I2cTransaction::write(0x09, vec![0x99, 0x02]),
            I2cTransaction::write(0x09, vec![0x98, 0x03]),
            I2cTransaction::write(0x09, vec![0x99, 0x0D]),
            I2cTransaction::write(0x09, vec![0x98, 0x04]),
            I2cTransaction::write(0x09, vec![0x99, 0x05]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x93, 20]),
            I2cTransaction::write_read(0x09, vec![0x01], vec![0x01, 0xEF]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P27::new(i2c, 0x09, MockNoop {}).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_radio_frequency() {
        let expectations = [
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x98, 0x07]),
            I2cTransaction::write(0x09, vec![0x99, 0xE5]),
            I2cTransaction::write(0x09, vec![0x98, 0x08]),
            I2cTransaction::write(0x09, vec![0x99, 0x61]),
            I2cTransaction::write(0x09, vec![0x98, 0x09]),
            I2cTransaction::write(0x09, vec![0x99, 0xd2]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        p27.set_radio_frequency(917_u32.MHz()).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_radio_frequency_too_small() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(
            p27.set_radio_frequency(889.MHz()),
            Err(Error::ArgumentError)
        );

        i2c_clone.done();
    }

    #[test]
    pub fn set_radio_frequency_too_large() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(
            p27.set_radio_frequency(1021.MHz()),
            Err(Error::ArgumentError)
        );

        i2c_clone.done();
    }

    #[test]
    pub fn set_bit_rate() {
        let expectations = [
            I2cTransaction::write(0x09, vec![0x98, 0x03]),
            I2cTransaction::write(0x09, vec![0x99, 0x0D]),
            I2cTransaction::write(0x09, vec![0x98, 0x04]),
            I2cTransaction::write(0x09, vec![0x99, 0x05]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        p27.set_bit_rate(9_600).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_bit_rate_too_large() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.set_bit_rate(u32::MAX), Err(Error::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn set_bit_rate_too_small() {
        let expectations = [];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.set_bit_rate(0), Err(Error::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn set_tx_power() {
        let expectations = [
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x93, 0x03]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        p27.set_tx_power(3).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn get_tx_power() {
        let expectations = [
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write_read(0x09, vec![0x13], vec![0xFF]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.get_tx_power(), Ok(-1));

        i2c_clone.done();
    }

    #[test]
    pub fn new_payload_true() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x23], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.new_payload(), Ok(true));

        i2c_clone.done();
    }

    #[test]
    pub fn new_payload_false() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x23], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.new_payload(), Ok(false));

        i2c_clone.done();
    }

    #[test]
    pub fn set_destination_radio_address() {
        let expectations = [I2cTransaction::write(0x09, vec![0x97, 0x00, 0x0C])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        p27.set_destination_radio_address(12).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn get_destination_radio_address() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x17], vec![0x00, 43])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.get_destination_radio_address(), Ok(43));

        i2c_clone.done();
    }

    #[test]
    pub fn send() {
        let expectations = [
            I2cTransaction::write(0x09, vec![0x97, 0x00, 0x1F]),
            I2cTransaction::write(0x09, vec![0xA1, 0x04]),
            I2cTransaction::write(0x09, vec![0xA2, 0x77, 0x61, 0x76, 0x65]),
            I2cTransaction::write(0x09, vec![0xA4, 0x01]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.send(31, b"wave"), Ok(()));

        i2c_clone.done();
    }

    #[test]
    pub fn send_error() {
        let i2c_error = ErrorKind::Other;
        let expectations =
            [I2cTransaction::write(0x09, vec![0x97, 0x00, 0x1F]).with_error(i2c_error)];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.send(31, b"wave"), Err(Error::I2cError(i2c_error)));

        i2c_clone.done();
    }

    #[test]
    pub fn receive_none() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x23], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        let mut data: [u8; 5] = [0; 5];
        assert_eq!(p27.receive(&mut data), Ok(0));
        assert_eq!(data, *b"\0\0\0\0\0");

        i2c_clone.done();
    }

    #[test]
    pub fn receive_some() {
        let expectations = [
            I2cTransaction::write_read(0x09, vec![0x23], vec![0x01]),
            I2cTransaction::write_read(0x09, vec![0x21], vec![0x03]),
            I2cTransaction::write_read(0x09, vec![0x22], vec![0x48, 0x69, 0x21]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        let mut data: [u8; 5] = [0; 5];
        assert_eq!(p27.receive(&mut data), Ok(3));
        assert_eq!(data, *b"Hi!\0\0");

        i2c_clone.done();
    }

    #[test]
    pub fn enable() {
        let expectations = [I2cTransaction::write(0x09, vec![0x14 | 0x80, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.enable(), Ok(()));

        i2c_clone.done();
    }

    #[test]
    pub fn disable() {
        let expectations = [I2cTransaction::write(0x09, vec![0x14 | 0x80, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.disable(), Ok(()));

        i2c_clone.done();
    }

    #[test]
    pub fn enabled_true() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x14], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.enabled(), Ok(true));

        i2c_clone.done();
    }

    #[test]
    pub fn enabled_false() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x14], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.enabled(), Ok(false));

        i2c_clone.done();
    }

    #[test]
    pub fn transceiver_ready_false() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x25], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.transceiver_ready(), Ok(false));

        i2c_clone.done();
    }

    #[test]
    pub fn transceiver_ready_true() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x25], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.transceiver_ready(), Ok(true));

        i2c_clone.done();
    }

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x01], vec![1, 2])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.whoami(), Ok(258));

        i2c_clone.done();
    }

    #[test]
    pub fn reset() {
        let expectations = [I2cTransaction::write(0x09, vec![0x20 | 0x80, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.reset(), Ok(()));

        i2c_clone.done();
    }

    #[test]
    pub fn firmware() {
        let expectations = [
            I2cTransaction::write_read(0x09, vec![0x02], vec![0x01]),
            I2cTransaction::write_read(0x09, vec![0x03], vec![0x02]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };

        assert_eq!(p27.firmware(), Ok((0x01, 0x02)));
        i2c_clone.done();
    }

    #[test]
    pub fn set_address() {
        let expectations = [I2cTransaction::write(0x09, vec![0x04, 0x69])];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        p27.set_address(0x69).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_small() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.set_address(0x07), Err(Error::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_large() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.set_address(0x78), Err(Error::ArgumentError));

        i2c_clone.done();
    }
}
