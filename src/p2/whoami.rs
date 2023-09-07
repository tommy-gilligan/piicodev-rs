use crate::{p2::P2, WhoAmI};
use embedded_hal::i2c::I2c;

const REG_WHOAMI: u8 = 0xD0;

impl<I2C: I2c> WhoAmI<I2C, u8> for P2<I2C> {
    const EXPECTED_WHOAMI: u8 = 0x60;

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
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::{p2::P2, WhoAmI};

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0xD0], vec![0xf0])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p2 = P2 {
            i2c,
            address: 0x09,
            temperature_data: None,
            pressure_data: None,
            humidity_data: None,
            t_fine: None,
        };
        assert_eq!(p2.whoami(), Ok(0xf0));

        i2c_clone.done();
    }
}
