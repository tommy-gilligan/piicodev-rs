use crate::{p27::P27, WhoAmI};
use embedded_hal::i2c::I2c;

const REG_WHOAMI: u8 = 0x01;

impl<I2C: I2c, DELAY> WhoAmI<I2C, u16> for P27<I2C, DELAY> {
    const EXPECTED_WHOAMI: u16 = 0x01EF;

    fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[REG_WHOAMI], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod whoami_test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;

    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::{p27::P27, WhoAmI};

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(0x09, vec![0x01], vec![1, 2])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p27 = P27 {
            i2c,
            delay: embedded_hal_mock::eh1::delay::NoopDelay {},
            address: 0x09,
        };
        assert_eq!(p27.whoami(), Ok(258));

        i2c_clone.done();
    }
}
