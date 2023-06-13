use crate::{p19::P19, WhoAmI};
use embedded_hal::i2c::I2c;

const REG_WHOAMI: u8 = 0x28;

impl<I2C: I2c> WhoAmI<I2C, u8> for P19<I2C> {
    const EXPECTED_WHOAMI: u8 = 0;

    fn whoami(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_WHOAMI], &mut data)?;
        Ok(data[0] >> 4_u8)
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::{p19::P19, WhoAmI};

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(0x52, vec![0x28], vec![201])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.whoami(), Ok(12));
        i2c_clone.done();
    }
}
