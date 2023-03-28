#![no_std]

use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::I2c;

#[derive(Copy, Clone)]
pub enum Address {
    X09 = 0x09,
}

pub struct P27<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
    address: Address,
}

const DEVICE_ID: u16 = 495;
const REG_WHOAMI: u8 = 0x01;
const REG_FIRM_MAJ: u8 = 0x02;
const REG_FIRM_MIN: u8 = 0x03;
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

impl<I2C: I2c, DELAY: DelayUs> P27<I2C, DELAY> {
    /// # Errors
    pub fn new(i2c: I2C, delay: DELAY, address: Address) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            delay,
            address,
        };

        res.set_led(true)?;

        res.i2c
            .write(res.address as u8, &[REG_RFM69_NODE_ID, 0, 0])?;
        while !(res.transceiver_ready()?) {
            res.delay.delay_ms(10);
        }
        res.i2c
            .write(res.address as u8, &[REG_RFM69_NETWORK_ID, 0, 0])?;

        res.set_radio_frequency(922)?;
        res.set_speed(2)?;
        res.set_tx_power(20)?;
        if res.whoami()? != DEVICE_ID {}
        Ok(res)
    }

    /// # Errors
    pub fn send(&mut self, _data: &[u8]) -> Result<(), I2C::Error> {
        let address = 0;
        self.set_destination_radio_address(address)?;
        self.delay.delay_ms(8);

        self.i2c
            .write(self.address as u8, &[REG_PAYLOAD_LENGTH, 3])?;
        self.delay.delay_ms(5);

        self.i2c
            .write(self.address as u8, &[REG_PAYLOAD, 0x42, 0x4f, 0x42])?;

        self.delay.delay_ms(5);
        self.i2c.write(self.address as u8, &[REG_PAYLOAD_GO, 1])?;
        Ok(())
    }

    /// # Errors
    pub fn payload_new(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address as u8, &[REG_PAYLOAD_NEW], &mut data)?;
        Ok(data[0] == 1)
    }

    /// # Errors
    pub fn get_destination_radio_address(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c
            .write_read(self.address as u8, &[REG_RFM69_TO_NODE_ID], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    /// # Errors
    pub fn set_destination_radio_address(&mut self, value: u8) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address as u8, &[REG_RFM69_TO_NODE_ID, value])?;
        Ok(())
    }

    /// # Errors
    pub fn get_rfm69_register(&mut self, register: u8) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write(self.address as u8, &[REG_RFM69_REG, register])?;
        self.i2c
            .write_read(self.address as u8, &[REG_RFM69_VALUE], &mut data)?;
        Ok(data[0])
    }

    /// # Errors
    pub fn set_rfm69_register(&mut self, register: u8, value: u8) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address as u8, &[REG_RFM69_REG, register])?;
        self.i2c
            .write(self.address as u8, &[REG_RFM69_VALUE, value])?;
        Ok(())
    }

    /// # Errors
    pub fn on(&mut self) -> Result<(), I2C::Error> {
        self.set_on()?;
        Ok(())
    }

    /// # Errors
    pub fn off(&mut self) -> Result<(), I2C::Error> {
        self.set_off()?;
        Ok(())
    }

    /// # Errors
    pub fn rfm69_reset(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(self.address as u8, &[REG_RFM69_RESET, 1])?;
        self.delay.delay_ms(10);
        Ok(())
    }

    /// # Errors
    pub fn get_speed(&mut self) -> Result<(u8, u8), I2C::Error> {
        self.delay.delay_ms(10);
        let msb = self.get_rfm69_register(RFM69_REG_BITRATEMSB)?;
        self.delay.delay_ms(10);
        let lsb = self.get_rfm69_register(RFM69_REG_BITRATELSB)?;
        self.delay.delay_ms(10);
        Ok((msb, lsb))
    }

    /// # Errors
    pub fn set_speed(&mut self, speed: u8) -> Result<(), I2C::Error> {
        let (msb, lsb) = match speed {
            1 => (0x0D, 0x05),
            2 => (0x01, 0x16),
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
    pub fn get_radio_frequency(&mut self) -> Result<(u8, u8, u8), I2C::Error> {
        self.delay.delay_ms(5);
        let msb = self.get_rfm69_register(RFM69_REG_FRFMSB)?;
        self.delay.delay_ms(5);
        let mid = self.get_rfm69_register(RFM69_REG_FRFMID)?;
        self.delay.delay_ms(5);
        let lsb = self.get_rfm69_register(RFM69_REG_FRFLSB)?;
        self.delay.delay_ms(5);
        Ok((msb, mid, lsb))
    }

    /// # Errors
    pub fn set_radio_frequency(&mut self, frequency: u16) -> Result<(), I2C::Error> {
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        let (msb, mid, lsb) = match frequency {
            915 => (0xE4, 0xC0, 0x00),
            918 => (0xE5, 0x80, 0x00),
            922 => (0xE6, 0xC0, 0x00),
            925 => (0xE7, 0x40, 0x00),
            928 => (0xE8, 0x00, 0x00),
            // should be an error
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
    pub fn get_tx_power(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        self.i2c
            .write_read(self.address as u8, &[REG_TX_POWER], &mut data)?;
        Ok(data[0])
    }

    /// # Errors
    pub fn set_tx_power(&mut self, _value: i8) -> Result<(), I2C::Error> {
        while !(self.transceiver_ready()?) {
            self.delay.delay_ms(10);
        }
        self.i2c.write(self.address as u8, &[REG_TX_POWER, 20])?;
        Ok(())
    }

    /// # Errors
    pub fn group(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address as u8, &[REG_RFM69_NETWORK_ID], &mut data)?;
        Ok(data[0])
    }

    /// # Errors
    pub fn radio_address(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
        self.i2c
            .write_read(self.address as u8, &[REG_RFM69_NODE_ID], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    /// # Errors
    pub fn get_on(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address as u8, &[REG_RFM69_RADIO_STATE], &mut data)?;
        self.delay.delay_ms(5);
        Ok(data[0] == 1)
    }

    /// # Errors
    pub fn set_on(&mut self) -> Result<(), I2C::Error> {
        self.delay.delay_ms(5);
        self.i2c
            .write(self.address as u8, &[REG_RFM69_RADIO_STATE, 1])?;
        self.delay.delay_ms(5);
        Ok(())
    }

    /// # Errors
    pub fn get_off(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address as u8, &[REG_RFM69_RADIO_STATE], &mut data)?;
        self.delay.delay_ms(5);
        Ok(data[0] == 0)
    }

    /// # Errors
    pub fn set_off(&mut self) -> Result<(), I2C::Error> {
        self.delay.delay_ms(5);
        self.i2c
            .write(self.address as u8, &[REG_RFM69_RADIO_STATE, 0])?;
        self.delay.delay_ms(5);
        Ok(())
    }

    /// # Errors
    pub fn get_led(&mut self) -> Result<bool, I2C::Error> {
        let mut maj: [u8; 1] = [0];
        self.i2c
            .write_read(self.address as u8, &[REG_LED], &mut maj)?;
        Ok(maj[0] == 1)
    }

    /// # Errors
    pub fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        if on {
            self.i2c.write(self.address as u8, &[REG_LED, 1])?;
        } else {
            self.i2c.write(self.address as u8, &[REG_LED, 0])?;
        }
        Ok(())
    }

    /// # Errors
    pub fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut maj: [u8; 2] = [0, 0];
        self.i2c
            .write_read(self.address as u8, &[REG_WHOAMI], &mut maj)?;
        Ok(u16::from_be_bytes(maj))
    }

    /// # Errors
    pub fn firmware(&mut self) -> Result<(u8, u8), I2C::Error> {
        let mut maj: [u8; 1] = [0];
        self.i2c
            .write_read(self.address as u8, &[REG_FIRM_MAJ], &mut maj)?;
        let mut min: [u8; 1] = [0];
        self.i2c
            .write_read(self.address as u8, &[REG_FIRM_MIN], &mut min)?;
        Ok((maj[0], min[0]))
    }

    /// # Errors
    pub fn transceiver_ready(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address as u8, &[REG_TRANSCEIVER_READY], &mut data)?;
        Ok(data[0] == 1)
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

    use crate::{Address, P27};

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x09, vec![0x05, 0x01]),
            I2cTransaction::write(0x09, vec![0x15, 0x00, 0x00]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x00]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x15, 0x16, 0x00]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x18, 0x07]),
            I2cTransaction::write(0x09, vec![0x19, 0xE4]),
            I2cTransaction::write(0x09, vec![0x18, 0x08]),
            I2cTransaction::write(0x09, vec![0x19, 0xC0]),
            I2cTransaction::write(0x09, vec![0x18, 0x09]),
            I2cTransaction::write(0x09, vec![0x19, 0x00]),
            I2cTransaction::write(0x09, vec![0x18, 0x03]),
            I2cTransaction::write(0x09, vec![0x19, 0x01]),
            I2cTransaction::write(0x09, vec![0x18, 0x04]),
            I2cTransaction::write(0x09, vec![0x19, 0x16]),
            I2cTransaction::write_read(0x09, vec![0x25], vec![0x01]),
            I2cTransaction::write(0x09, vec![0x13, 20]),
            I2cTransaction::write_read(0x09, vec![0x01], vec![0x01, 0xEF]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P27::new(i2c, MockNoop {}, Address::X09).unwrap();

        i2c_clone.done();
    }
}
