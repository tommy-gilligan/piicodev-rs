use crate::{p18::P18, Atmel, SetAddressError};
use embedded_hal::i2c::I2c;

const REG_FIRM_MAJ: u8 = 0x02;
const REG_FIRM_MIN: u8 = 0x03;
const REG_I2C_ADDRESS: u8 = 0x04;
const REG_LED: u8 = 0x07;

impl<I2C: I2c> Atmel<I2C> for P18<I2C> {
    // broken
    fn get_led(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0; 1];
        self.i2c.write_read(self.address, &[REG_LED], &mut data)?;
        if data[0] == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        if on {
            self.i2c.write(self.address, &[REG_LED, 1])?;
        } else {
            self.i2c.write(self.address, &[REG_LED, 0])?;
        }
        Ok(())
    }

    fn firmware(&mut self) -> Result<(u8, u8), I2C::Error> {
        let mut maj_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_FIRM_MAJ], &mut maj_data)?;
        let mut min_data: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[REG_FIRM_MIN], &mut min_data)?;
        Ok((maj_data[0], min_data[0]))
    }

    fn set_address(&mut self, new_address: u8) -> Result<(), SetAddressError<I2C::Error>> {
        if !(0x08..=0x77).contains(&new_address) {
            return Err(SetAddressError::ArgumentError);
        }
        self.i2c
            .write(self.address, &[REG_I2C_ADDRESS, new_address])
            .map_err(SetAddressError::I2cError)?;
        Ok(())
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod atmel_test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::{p18::P18, Atmel, SetAddressError};

    #[test]
    pub fn set_led_on() {
        let expectations = [I2cTransaction::write(0x5C, vec![0x07, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.set_led(true), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_led_off() {
        let expectations = [I2cTransaction::write(0x5C, vec![0x07, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.set_led(false), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_off() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x07], vec![0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.get_led(), Ok(false));
        i2c_clone.done();
    }

    #[test]
    pub fn get_led_on() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x07], vec![0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.get_led(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn firmware() {
        let expectations = [
            I2cTransaction::write_read(0x5C, vec![0x02], vec![0x01]),
            I2cTransaction::write_read(0x5C, vec![0x03], vec![0x02]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.firmware(), Ok((0x01, 0x02)));
        i2c_clone.done();
    }

    #[test]
    pub fn set_address() {
        let expectations = [I2cTransaction::write(0x09, vec![0x04, 0x69])];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p18 = P18 { i2c, address: 0x09 };
        p18.set_address(0x69).unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_small() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p18 = P18 { i2c, address: 0x09 };
        assert_eq!(p18.set_address(0x07), Err(SetAddressError::ArgumentError));

        i2c_clone.done();
    }

    #[test]
    pub fn set_address_too_large() {
        let expectations = [];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();
        let mut p18 = P18 { i2c, address: 0x09 };
        assert_eq!(p18.set_address(0x78), Err(SetAddressError::ArgumentError));

        i2c_clone.done();
    }
}
