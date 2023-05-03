#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![no_std]
use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::I2c;

pub struct P27<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
    address: u8,
}

const DEVICE_ID: u16 = 495;
const REG_WHOAMI: u8 = 0x01;
const REG_TX_POWER: u8 = 0x13;
const REG_RFM69_TO_NODE_ID: u8 = 0x17;
const REG_RFM69_REG: u8 = 0x18;
const REG_RFM69_VALUE: u8 = 0x19;
const REG_PAYLOAD_NEW: u8 = 0x23;
const REG_TRANSCEIVER_READY: u8 = 0x25;
const RFM69_REG_BITRATEMSB: u8 = 0x03;
const RFM69_REG_BITRATELSB: u8 = 0x04;
const RFM69_REG_FRFMSB: u8 = 0x07;
const RFM69_REG_FRFMID: u8 = 0x08;
const RFM69_REG_FRFLSB: u8 = 0x09;

const SET_REG_RFM69_NODE_ID: u8 = 0x95;
const SET_REG_RFM69_NETWORK_ID: u8 = 0x96;
const SET_REG_RFM69_TO_NODE_ID: u8 = 0x97;
const SET_REG_RFM69_REG: u8 = 0x98;
const SET_REG_RFM69_VALUE: u8 = 0x99;
const SET_REG_LED: u8 = 0x85;
const SET_REG_TX_POWER: u8 = 0x93;

const REG_PAYLOAD_LENGTH: u8 = 0x21;
const REG_PAYLOAD: u8 = 0x22;

const SET_REG_PAYLOAD_LENGTH: u8 = 0xA1;
const SET_REG_PAYLOAD: u8 = 0xA2;
const SET_REG_PAYLOAD_GO: u8 = 0xA4;

impl<I2C: I2c, DELAY: DelayUs> P27<I2C, DELAY> {
    /// # Errors
    pub fn new(i2c: I2C, address: u8, delay: DELAY) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            delay,
            address,
        };

        res.set_led(true)?;

        res.i2c.write(res.address, &[SET_REG_RFM69_NODE_ID, 0, 0])?;
        while !(res.transceiver_ready()?) {
            res.delay.delay_ms(10);
        }
        res.i2c.write(res.address, &[SET_REG_RFM69_NETWORK_ID, 0])?;

        res.set_radio_frequency(922)?;
        res.set_speed(1)?;

        res.set_tx_power(20)?;
        if res.whoami()? != DEVICE_ID {}
        Ok(res)
    }

    /// # Errors
    pub fn send(&mut self, address: u16, data: &[u8]) -> Result<(), I2C::Error> {
        self.set_destination_radio_address(address)?;
        self.i2c.write(
            self.address,
            &[SET_REG_PAYLOAD_LENGTH, data.len().try_into().unwrap()],
        )?;
        self.delay.delay_ms(5);

        let mut send_data: [u8; 33] = [SET_REG_PAYLOAD; 33];
        for (i, _c) in data.iter().enumerate() {
            send_data[i + 1] = data[i]
        }
        self.i2c
            .write(self.address, &send_data[0..(data.len() + 1)])?;
        self.delay.delay_ms(5);

        self.i2c.write(self.address, &[SET_REG_PAYLOAD_GO, 1])?;
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
        self.i2c
            .write(self.address, &[SET_REG_RFM69_TO_NODE_ID, data[0], data[1]])?;
        Ok(())
    }

    /// # Errors
    pub fn get_rfm69_register(&mut self, register: u8) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c.write(self.address, &[REG_RFM69_REG, register])?;
        self.i2c
            .write_read(self.address, &[REG_RFM69_VALUE], &mut data)?;
        Ok(data[0])
    }

    /// # Errors
    pub fn set_rfm69_register(&mut self, register: u8, value: u8) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[SET_REG_RFM69_REG, register])?;
        self.i2c
            .write(self.address, &[SET_REG_RFM69_VALUE, value])?;
        Ok(())
    }

    /// # Errors
    pub fn get_speed(&mut self) -> Result<u8, I2C::Error> {
        self.delay.delay_ms(10);
        let msb = self.get_rfm69_register(RFM69_REG_BITRATEMSB)?;
        self.delay.delay_ms(10);
        let lsb = self.get_rfm69_register(RFM69_REG_BITRATELSB)?;
        self.delay.delay_ms(10);

        let speed = match (msb, lsb) {
            (0x0D, 0x05) => 1,
            (0x00, 0x6B) => 3,
            _ => 2,
        };

        Ok(speed)
    }

    /// # Errors
    pub fn set_speed(&mut self, speed: u8) -> Result<(), I2C::Error> {
        let (msb, lsb) = match speed {
            1 => (0x0D, 0x05),
            3 => (0x00, 0x6B),
            _ => (0x01, 0x16),
        };

        self.delay.delay_ms(10);
        self.set_rfm69_register(RFM69_REG_BITRATEMSB, msb)?;
        self.delay.delay_ms(10);
        self.set_rfm69_register(RFM69_REG_BITRATELSB, lsb)?;
        self.delay.delay_ms(10);
        Ok(())
    }

    /// # Errors
    pub fn get_radio_frequency(&mut self) -> Result<u16, I2C::Error> {
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        self.delay.delay_ms(5);
        let msb = self.get_rfm69_register(RFM69_REG_FRFMSB)?;
        self.delay.delay_ms(5);
        let mid = self.get_rfm69_register(RFM69_REG_FRFMID)?;
        self.delay.delay_ms(5);
        let lsb = self.get_rfm69_register(RFM69_REG_FRFLSB)?;
        self.delay.delay_ms(5);

        let frequency = match (msb, mid, lsb) {
            (0xE5, 0x80, 0x00) => 918,
            (0xE6, 0xC0, 0x00) => 922,
            (0xE7, 0x40, 0x00) => 925,
            (0xE8, 0x00, 0x00) => 928,
            _ => 915,
        };

        Ok(frequency)
    }

    /// # Errors
    pub fn set_radio_frequency(&mut self, frequency: u16) -> Result<(), I2C::Error> {
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        let (msb, mid, lsb) = match frequency {
            918 => (0xE5, 0x80, 0x00),
            922 => (0xE6, 0x80, 0x00),
            925 => (0xE7, 0x40, 0x00),
            928 => (0xE8, 0x00, 0x00),
            _ => (0xE4, 0xC0, 0x00),
        };

        self.delay.delay_ms(5);
        self.set_rfm69_register(RFM69_REG_FRFMSB, msb)?;
        self.delay.delay_ms(5);
        self.set_rfm69_register(RFM69_REG_FRFMID, mid)?;
        self.delay.delay_ms(5);
        self.set_rfm69_register(RFM69_REG_FRFLSB, lsb)?;
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

        #[allow(clippy::cast_possible_wrap)]
        Ok(data[0] as i8)
    }

    /// # Errors
    pub fn set_tx_power(&mut self, value: i8) -> Result<(), I2C::Error> {
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        #[allow(clippy::cast_sign_loss)]
        self.i2c
            .write(self.address, &[SET_REG_TX_POWER, value as u8])?;
        Ok(())
    }

    /// # Errors
    pub fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        if on {
            self.i2c.write(self.address, &[SET_REG_LED, 1])?;
        } else {
            self.i2c.write(self.address, &[SET_REG_LED, 0])?;
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

    use crate::P27;

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
            I2cTransaction::write(0x09, vec![0x99, 0x80]),
            I2cTransaction::write(0x09, vec![0x98, 0x09]),
            I2cTransaction::write(0x09, vec![0x99, 0x00]),
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
            I2cTransaction::write(0x09, vec![0x99, 0x80]),
            I2cTransaction::write(0x09, vec![0x98, 0x09]),
            I2cTransaction::write(0x09, vec![0x99, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        p27.set_radio_frequency(918).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn get_radio_frequency() {
        let expectations = [
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x18, 0x07]),
            I2cTransaction::write_read(0x09, vec![0x19], vec![0xE7]),
            I2cTransaction::write(0x09, vec![0x18, 0x08]),
            I2cTransaction::write_read(0x09, vec![0x19], vec![0x40]),
            I2cTransaction::write(0x09, vec![0x18, 0x09]),
            I2cTransaction::write_read(0x09, vec![0x19], vec![0x00]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        p27.get_radio_frequency().unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_speed() {
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
        p27.set_speed(1).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn get_speed() {
        let expectations = [
            I2cTransaction::write(0x09, vec![0x18, 0x03]),
            I2cTransaction::write_read(0x09, vec![0x19], vec![0x00]),
            I2cTransaction::write(0x09, vec![0x18, 0x04]),
            I2cTransaction::write_read(0x09, vec![0x19], vec![0x6B]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: MockNoop {},
            address: 0x09,
        };
        assert_eq!(p27.get_speed(), Ok(3));

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
}
