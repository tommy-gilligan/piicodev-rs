use crate::{p1::P1, WhoAmI};
use embedded_hal::i2c::I2c;

const REG_WHOAMI: u8 = 0x0F;

impl<I2C: I2c> WhoAmI<I2C, u16> for P1<I2C> {
    const EXPECTED_WHOAMI: u16 = 0x0117;

    fn whoami(&mut self) -> Result<u16, I2C::Error> {
        let mut data: [u8; 2] = [0, 0];
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
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::{p1::P1, WhoAmI};

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(
            0x09,
            vec![0x0F],
            vec![0xf0, 0x0d],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p1 = P1 { i2c, address: 0x09 };
        assert_eq!(p1.whoami(), Ok(0xf00d));

        i2c_clone.done();
    }
}
