use crate::{p18::P18, WhoAmI};
use embedded_hal::i2c::I2c;

const REG_WHOAMI: u8 = 0x11;

impl<I2C: I2c> WhoAmI<I2C, u8> for P18<I2C> {
    const EXPECTED_WHOAMI: u8 = 0x51;

    fn whoami(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_WHOAMI], &mut data)?;
        Ok(data[0])
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod whoami_test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use crate::{p18::P18, WhoAmI};
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(0x5C, vec![0x11], vec![0x23])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p18 = P18 { i2c, address: 0x5C };

        assert_eq!(p18.whoami(), Ok(0x23));
        i2c_clone.done();
    }
}
