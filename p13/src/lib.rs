#![doc = include_str!("../README.md")]
#![no_std]
#![feature(lint_reasons)]

use embedded_hal::i2c::I2c;
use smart_leds_trait::SmartLedsWrite;

pub struct P13<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> P13<I2C> {
    pub const fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<I2C: I2c> SmartLedsWrite for P13<I2C> {
    type Color = smart_leds_trait::RGB8;
    type Error = I2C::Error;

    /// # Errors
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        let mut data: [u8; 10] = [0x07, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];
        for (i, v) in iterator
            .flat_map(|item| {
                let color: Self::Color = item.into();
                [color.r, color.g, color.b]
            })
            .enumerate()
        {
            data[i + 1] = v;
        }
        self.i2c.write(self.address, &data)?;

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
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use smart_leds_trait::{SmartLedsWrite, RGB};

    use crate::P13;

    #[test]
    pub fn write() {
        let expectations = [I2cTransaction::write(
            0x0A,
            vec![0x07, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p13 = P13::new(i2c, 0x0A);

        let data: [RGB<u8>; 3] = [
            RGB {
                r: 0xff,
                g: 0x00,
                b: 0x00,
            },
            RGB {
                r: 0xff,
                g: 0xff,
                b: 0x00,
            },
            RGB {
                r: 0xff,
                g: 0x00,
                b: 0xff,
            },
        ];
        p13.write(data.iter().copied()).unwrap();
        i2c_clone.done();
    }
}
